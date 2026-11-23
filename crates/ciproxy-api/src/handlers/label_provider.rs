//! Provider fit from each trusted catalog, with temporary holds kept separate.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_overflow::view::{OverflowView, ProviderView, RefreshStatus};
use ciproxy_protocol::resources::VmSize;
use ciproxy_scaler::size_policy::{ProviderChoice, select_provider_size};
use ciproxy_scaler::snapshot::NothingFits;
use ciproxy_store_interface::overflow::provider::OverflowProvider;

use crate::wire::job_fit::{ProviderCatalogSize, ProviderHold, ProviderLabelFit, ProviderSizeFit};
use crate::wire::jobs::JobVmSize;

pub(super) struct Observation<'a> {
    pub(super) view: Option<&'a OverflowView>,
    pub(super) roots: Option<&'a BTreeMap<OverflowProvider, RootSizing>>,
    pub(super) image_root_bytes: u64,
    pub(super) enabled: bool,
    pub(super) at: DateTime<Utc>,
}

pub(super) struct ProviderFit {
    pub(super) view: ProviderLabelFit,
    pub(super) largest_holds_image: bool,
    pub(super) largest_refused: bool,
}

pub(super) fn fit(
    provider: OverflowProvider,
    requested: VmSize,
    allowed: bool,
    allows_fallback: bool,
    observation: &Observation<'_>,
) -> ProviderFit {
    let observed = observation.view.and_then(|view| view.provider(provider));
    let current = observed.filter(|view| view.refresh == RefreshStatus::Current);
    let sizing = match observation.roots {
        Some(roots) => roots.get(&provider).copied(),
        None => Some(RootSizing {
            image_root_bytes: observation.image_root_bytes,
        }),
    };
    let required = sizing.and_then(|root| root.vm_size(requested));
    let catalog = current.and_then(|view| view.resource_catalog().ok().flatten());
    let largest = catalog.as_ref().and_then(|catalog| {
        catalog
            .sizes
            .iter()
            .find(|size| Some(&size.id) == catalog.largest_size.as_ref())
    });
    let refused =
        |name: &String| current.is_some_and(|view| view.state.refused_sizes.contains(name));
    let mut eligible_size = None;
    let fit = match (allowed, required, catalog.as_ref()) {
        (false, _, _) => ProviderSizeFit::Disabled,
        (true, Some(required), Some(catalog)) => {
            eligible_size = match select_provider_size(
                catalog.sizes.iter().filter(|size| !refused(&size.id.0)),
                None,
                required,
                0,
                false,
                NothingFits::Wait,
            ) {
                Some(ProviderChoice::Fitting(size)) => Some(ProviderCatalogSize {
                    name: size.id.0.clone(),
                    size: size_view(size.size),
                }),
                _ => None,
            };
            match select_provider_size(
                catalog.sizes.iter(),
                None,
                required,
                0,
                false,
                NothingFits::Wait,
            ) {
                Some(ProviderChoice::Fitting(size)) => ProviderSizeFit::Fitting {
                    size_name: size.id.0.clone(),
                    size: size_view(size.size),
                    refused: refused(&size.id.0),
                },
                _ => ProviderSizeFit::NoSizeFits,
            }
        }
        _ => ProviderSizeFit::Unavailable,
    };
    let largest_holds_image = allowed
        && required.is_some()
        && largest
            .zip(sizing)
            .is_some_and(|(size, root)| size.size.disk_bytes >= root.image_root_bytes);
    let largest_refused = largest.is_some_and(|size| refused(&size.id.0));
    let candidate = eligible_size
        .as_ref()
        .map(|size| size.name.as_str())
        .or_else(|| {
            largest
                .filter(|_| {
                    allows_fallback
                        && largest_holds_image
                        && !largest_refused
                        && matches!(fit, ProviderSizeFit::NoSizeFits)
                })
                .map(|size| size.id.0.as_str())
        });
    let hold = allowed
        .then(|| hold(observation, observed, candidate))
        .flatten();
    ProviderFit {
        view: ProviderLabelFit {
            provider: provider.into(),
            connected: current.map(|view| view.account.is_some()),
            fit,
            eligible_size,
            largest_size: largest.map(|size| ProviderCatalogSize {
                name: size.id.0.clone(),
                size: size_view(size.size),
            }),
            required_root_disk_bytes: required.map(|size| size.disk_bytes.to_string()),
            hold,
        },
        largest_holds_image,
        largest_refused,
    }
}

fn hold(
    observation: &Observation<'_>,
    provider: Option<&ProviderView>,
    size: Option<&str>,
) -> Option<ProviderHold> {
    if !observation.enabled {
        return Some(ProviderHold::Off);
    }
    let Some(provider) = provider else {
        return Some(ProviderHold::Unavailable);
    };
    let Some(account) = &provider.account else {
        return Some(ProviderHold::NotConnected);
    };
    if provider.refresh != RefreshStatus::Current {
        return Some(ProviderHold::Unavailable);
    }
    if account.rejected_at.is_some() {
        return Some(ProviderHold::KeyRejected);
    }
    match provider.size_limit_reached(size) {
        Some(true) => return Some(ProviderHold::SpendLimit),
        None => return Some(ProviderHold::CostsUnknown),
        Some(false) => {}
    }
    if provider
        .state
        .retry_at
        .is_some_and(|at| at > observation.at)
    {
        return Some(ProviderHold::RetryAfter);
    }
    if provider.state.billing_required {
        return Some(ProviderHold::BillingRequired);
    }
    None
}

fn size_view(size: VmSize) -> JobVmSize {
    JobVmSize {
        vcpus: size.vcpu,
        memory_bytes: size.memory_bytes.to_string(),
        disk_bytes: size.disk_bytes.to_string(),
    }
}
