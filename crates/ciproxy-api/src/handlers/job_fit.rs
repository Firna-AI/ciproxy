//! Waiting-label fit through the existing machine, image, and scaler owners.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_gateway::types::MachineView;
use ciproxy_overflow::view::{OverflowView, Spend};
use ciproxy_protocol::resources::{GIGABYTE, VmSize};
use ciproxy_scaler::boat::{BoatSize, BoatSizeId};
use ciproxy_scaler::size_policy::{BoatChoice, select_boat_size};
use ciproxy_scaler::snapshot::NothingFits;
use ciproxy_store_interface::job_views::DashboardJob;
use ciproxy_store_interface::jobs::JobState;
use ciproxy_store_interface::labels::{Label, WhenNothingFits};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::capacity::MachineCapacity;
use crate::wire::job_fit::{BoatLabelFit, WaitingLabelFit};
use crate::wire::jobs::JobVmSize;

use super::label_views;

pub(super) struct FitInput<'a> {
    pub jobs: &'a [DashboardJob],
    pub labels: Vec<Label>,
    pub machines: &'a [MachineView],
    pub capacity: &'a dyn MachineCapacity,
    pub overflow: Option<&'a OverflowView>,
    pub sizing: Option<RootSizing>,
    pub at: DateTime<Utc>,
    pub admin: bool,
}

pub(super) fn labels(input: FitInput<'_>) -> Vec<WaitingLabelFit> {
    let FitInput {
        jobs,
        labels,
        machines,
        capacity,
        overflow,
        sizing,
        at,
        admin,
    } = input;
    let mut waiting: BTreeMap<Uuid, (String, u64)> = BTreeMap::new();
    for job in jobs.iter().filter(|job| job.state == JobState::Queued) {
        let entry = waiting
            .entry(job.label_id)
            .or_insert((job.label.clone(), 0));
        entry.1 += 1;
    }
    let catalog = physical_catalog(overflow, at);
    waiting
        .into_iter()
        .map(|(id, (name, count))| {
            let label = labels.iter().find(|label| label.id == id);
            let current = label.map(|label| label_views::label(label.clone(), machines, capacity));
            let own_fits = current
                .as_ref()
                .is_some_and(|current| current.fit.fitting_machines > 0);
            let own_unknown = current.as_ref().is_none_or(|current| {
                current.label.allow_machines && current.fit.unavailable_machines > 0
            });
            let boat = match (label, sizing, catalog.as_ref(), overflow) {
                (Some(label), _, _, _) if !label.allow_boat => BoatLabelFit::Disabled,
                (Some(label), Some(sizing), Some(catalog), Some(view)) => boat_fit(
                    label,
                    sizing,
                    catalog,
                    &view.state.refused_sizes,
                    own_fits,
                    own_unknown,
                ),
                _ => BoatLabelFit::Unavailable,
            };
            let nothing_fits = if own_fits
                || matches!(
                    boat,
                    BoatLabelFit::Fitting { .. } | BoatLabelFit::LargestFallback { .. }
                ) {
                Some(false)
            } else if !own_unknown
                && matches!(boat, BoatLabelFit::NoSizeFits | BoatLabelFit::Disabled)
            {
                Some(true)
            } else {
                None
            };
            WaitingLabelFit {
                label_id: id.to_string(),
                label_name: name,
                waiting_jobs: count.to_string(),
                can_edit: admin && current.is_some(),
                ask_admin: !admin && nothing_fits == Some(true),
                current,
                boat,
                nothing_fits,
            }
        })
        .collect()
}

struct ResourceCatalog {
    sizes: Vec<BoatSize>,
    largest_size: Option<BoatSizeId>,
}

/// Temporary refusals and financial gaps do not change physical resource fit.
fn physical_catalog(overflow: Option<&OverflowView>, at: DateTime<Utc>) -> Option<ResourceCatalog> {
    let overflow = overflow?;
    overflow.account.as_ref()?;
    overflow.state.checked_at?;
    let mut physical = overflow.clone();
    physical.state.refused_sizes.clear();
    physical.spend = Spend {
        confirmed_usd: Decimal::ZERO,
        pending_usd: Decimal::ZERO,
        settled_bound_usd: Decimal::ZERO,
        unbounded_starts: Vec::new(),
        unknown_start_charges: Vec::new(),
        unmeasured_runtime: Vec::new(),
    };
    physical.limit_usd = None;
    let catalog = physical.scaler_account(at).ok()?;
    Some(ResourceCatalog {
        sizes: catalog.sizes,
        largest_size: catalog.largest_size,
    })
}

fn boat_fit(
    label: &Label,
    sizing: RootSizing,
    catalog: &ResourceCatalog,
    refused: &[String],
    own_fits: bool,
    own_unknown: bool,
) -> BoatLabelFit {
    let requested = VmSize {
        vcpu: label.size.vcpus.try_into().unwrap_or(0),
        memory_bytes: u64::try_from(label.size.memory_gb)
            .unwrap_or(0)
            .saturating_mul(GIGABYTE),
        disk_bytes: u64::try_from(label.size.disk_gb)
            .unwrap_or(0)
            .saturating_mul(GIGABYTE),
    };
    let Some(required) = sizing.vm_size(requested) else {
        return BoatLabelFit::Unavailable;
    };
    let policy = match label.when_nothing_fits {
        WhenNothingFits::Wait => NothingFits::Wait,
        WhenNothingFits::LargestBoat => NothingFits::LargestBoatSize,
    };
    match select_boat_size(
        catalog.sizes.iter(),
        catalog.largest_size.as_ref(),
        required,
        sizing.image_root_bytes,
        own_fits,
        policy,
    ) {
        Some(BoatChoice::Fitting(size)) => BoatLabelFit::Fitting {
            size_name: size.id.0.clone(),
            size: size_view(size.size),
            refused: refused.contains(&size.id.0),
        },
        Some(BoatChoice::Largest(_)) if own_unknown => BoatLabelFit::Unavailable,
        Some(BoatChoice::Largest(size)) => BoatLabelFit::LargestFallback {
            size_name: size.id.0.clone(),
            size: size_view(size.size),
            refused: refused.contains(&size.id.0),
        },
        None => BoatLabelFit::NoSizeFits,
    }
}

fn size_view(size: VmSize) -> JobVmSize {
    JobVmSize {
        vcpus: size.vcpu,
        memory_bytes: size.memory_bytes.to_string(),
        disk_bytes: size.disk_bytes.to_string(),
    }
}
