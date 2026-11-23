//! Ordered capacity choices and the shared wait reason for Labels and Jobs.

use ciproxy_protocol::resources::VmSize;
use ciproxy_store_interface::overflow::provider::OverflowProvider;

use crate::wire::job_fit::{
    LabelFit, LabelRouting, LabelWaitReason, ProviderHold, ProviderSizeFit,
};
use crate::wire::settings::{LabelInput, LabelPlace, NothingFits};

use super::label_provider::{self, Observation, ProviderFit};

pub(super) fn enrich(
    fit: &mut LabelFit,
    label: &LabelInput,
    requested: VmSize,
    observation: &Observation<'_>,
) {
    let own_fits = fit.fitting_machines > 0;
    let own_unknown =
        label.order.contains(&LabelPlace::Machine) && !own_fits && fit.unavailable_machines > 0;
    let ordered = label.order.iter().filter_map(|place| match place {
        LabelPlace::Machine => None,
        LabelPlace::Overflow { provider } => Some(provider.0),
    });
    let all = ordered.clone().map(|provider| (provider, true)).chain(
        OverflowProvider::all()
            .filter(|provider| !ordered.clone().any(|listed| listed == *provider))
            .map(|provider| (provider, false)),
    );
    let allows_fallback = matches!(label.spec.when_nothing_fits, NothingFits::LargestOverflow);
    let mut providers: Vec<_> = all
        .map(|(provider, allowed)| {
            label_provider::fit(provider, requested, allowed, allows_fallback, observation)
        })
        .collect();
    fit.routing = routing(label, &providers, own_fits, own_unknown);
    record_physical_fallback(label, &mut providers, own_fits, own_unknown);
    for provider in &mut providers {
        if provider.view.hold.is_none() {
            provider.view.hold = size_hold(provider);
        }
    }
    let available = !own_unknown
        && providers.iter().all(|provider| {
            !matches!(provider.view.fit, ProviderSizeFit::Unavailable)
                || provider.view.connected == Some(false)
        });
    fit.fitting_places = available.then(|| {
        places_matching(label, &providers, own_fits, |provider| {
            matches!(
                provider.view.fit,
                ProviderSizeFit::Fitting { .. } | ProviderSizeFit::LargestFallback { .. }
            )
        })
    });
    fit.providers = providers
        .into_iter()
        .map(|provider| provider.view)
        .collect();
}

fn routing(
    label: &LabelInput,
    providers: &[ProviderFit],
    own_fits: bool,
    own_unknown: bool,
) -> LabelRouting {
    let waits = |reason| LabelRouting::Waits { reason };
    if label.order.is_empty() {
        return waits(LabelWaitReason::EmptyOrder);
    }
    let runnable = places_matching(label, providers, own_fits, |provider| {
        provider.view.eligible_size.is_some() && provider.view.hold.is_none()
    });
    if !runnable.is_empty() {
        return LabelRouting::Places { places: runnable };
    }
    let held = |provider: &ProviderFit, hold| LabelWaitReason::Provider {
        provider: provider.view.provider,
        hold,
    };
    if matches!(label.spec.when_nothing_fits, NothingFits::LargestOverflow)
        && !own_unknown
        && !providers
            .iter()
            .any(|provider| provider.view.eligible_size.is_some())
    {
        let mut candidates = providers
            .iter()
            .filter(|provider| provider.largest_holds_image);
        if let Some(provider) = candidates
            .clone()
            .find(|provider| provider.view.hold.is_none() && !provider.largest_refused)
        {
            return LabelRouting::Largest {
                provider: provider.view.provider,
            };
        }
        if let Some(provider) = candidates.next() {
            return waits(held(
                provider,
                provider.view.hold.unwrap_or(ProviderHold::SizeRefused),
            ));
        }
    }
    if let Some(reason) = providers.iter().find_map(|provider| {
        let hold = provider.view.hold.or_else(|| size_hold(provider))?;
        (matches!(provider.view.fit, ProviderSizeFit::Fitting { .. })
            || (matches!(provider.view.fit, ProviderSizeFit::Unavailable)
                && matches!(
                    hold,
                    ProviderHold::Off | ProviderHold::NotConnected | ProviderHold::Unavailable
                )))
        .then(|| held(provider, hold))
    }) {
        return waits(reason);
    }
    let unknown = own_unknown
        || providers
            .iter()
            .any(|provider| matches!(provider.view.fit, ProviderSizeFit::Unavailable));
    waits(if unknown {
        LabelWaitReason::Unavailable
    } else {
        LabelWaitReason::NothingFits
    })
}

fn places_matching(
    label: &LabelInput,
    providers: &[ProviderFit],
    own_fits: bool,
    matches: impl Fn(&ProviderFit) -> bool,
) -> Vec<LabelPlace> {
    label
        .order
        .iter()
        .copied()
        .filter(|place| match place {
            LabelPlace::Machine => own_fits,
            LabelPlace::Overflow { provider } => providers
                .iter()
                .any(|fit| fit.view.provider == *provider && matches(fit)),
        })
        .collect()
}

fn record_physical_fallback(
    label: &LabelInput,
    providers: &mut [ProviderFit],
    own_fits: bool,
    own_unknown: bool,
) {
    if !matches!(label.spec.when_nothing_fits, NothingFits::LargestOverflow)
        || own_fits
        || providers
            .iter()
            .any(|provider| matches!(provider.view.fit, ProviderSizeFit::Fitting { .. }))
    {
        return;
    }
    let Some(provider) = providers
        .iter_mut()
        .find(|provider| provider.largest_holds_image)
    else {
        return;
    };
    if own_unknown {
        provider.view.fit = ProviderSizeFit::Unavailable;
    } else if let Some(largest) = &provider.view.largest_size {
        provider.view.fit = ProviderSizeFit::LargestFallback {
            size_name: largest.name.clone(),
            size: largest.size.clone(),
            refused: provider.largest_refused,
        };
    }
}

fn size_hold(provider: &ProviderFit) -> Option<ProviderHold> {
    match provider.view.fit {
        ProviderSizeFit::Fitting { .. } if provider.view.eligible_size.is_none() => {
            Some(ProviderHold::SizeRefused)
        }
        ProviderSizeFit::LargestFallback { refused: true, .. } => Some(ProviderHold::SizeRefused),
        _ => None,
    }
}
