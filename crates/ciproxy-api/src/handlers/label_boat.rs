//! Physical Boat choices use the scaler policy and preserve temporary refusals.

use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_overflow::view::OverflowView;
use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::resources::{GIGABYTE, VmSize};
use ciproxy_scaler::size_policy::{ProviderChoice, select_provider_size};
use ciproxy_scaler::snapshot;

use crate::wire::facts::{LabelBoatSize, LabelPlacement};
use crate::wire::job_fit::BoatLabelFit;
use crate::wire::jobs::JobVmSize;
use crate::wire::settings::{LabelFit, LabelInput, NothingFits};

pub(super) fn enrich(
    fit: &mut LabelFit,
    label: &LabelInput,
    overflow: Option<&OverflowView>,
    sizing: RootSizing,
) {
    fit.boat_connected = overflow.map(|view| view.account.is_some());
    let requested = VmSize {
        vcpu: label.vcpus.try_into().unwrap_or(0),
        memory_bytes: u64::try_from(label.memory_gb)
            .unwrap_or(0)
            .saturating_mul(GIGABYTE),
        disk_bytes: u64::try_from(label.disk_gb)
            .unwrap_or(0)
            .saturating_mul(GIGABYTE),
    };
    let required = sizing.vm_size(requested);
    fit.boat_required_root_disk_bytes = required.map(|size| size.disk_bytes.to_string());
    let catalog = overflow.and_then(|view| view.resource_catalog().ok().flatten());
    fit.largest_boat = catalog
        .as_ref()
        .and_then(|catalog| {
            catalog
                .sizes
                .iter()
                .find(|size| Some(&size.id) == catalog.largest_size.as_ref())
        })
        .map(|size| LabelBoatSize {
            name: size.id.0.clone(),
            size: size_view(size.size),
        });
    let own_fits = fit.fitting_machines > 0;
    let own_unknown = label.allow_machines && fit.unavailable_machines > 0;
    let allow_boat = label.allow_boat && label.platform == Platform::LinuxX64;
    fit.boat = match (allow_boat, required, catalog, overflow) {
        (false, _, _, _) => BoatLabelFit::Disabled,
        (true, Some(required), Some(catalog), Some(view)) => {
            let policy = match label.when_nothing_fits {
                NothingFits::Wait => snapshot::NothingFits::Wait,
                NothingFits::LargestBoat => snapshot::NothingFits::LargestProviderSize,
            };
            match select_provider_size(
                catalog.sizes.iter(),
                catalog.largest_size.as_ref(),
                required,
                sizing.image_root_bytes,
                own_fits,
                policy,
            ) {
                Some(ProviderChoice::Fitting(size)) => BoatLabelFit::Fitting {
                    size_name: size.id.0.clone(),
                    size: size_view(size.size),
                    refused: view.state.refused_sizes.contains(&size.id.0),
                },
                Some(ProviderChoice::Largest(_)) if own_unknown => BoatLabelFit::Unavailable,
                Some(ProviderChoice::Largest(size)) => BoatLabelFit::LargestFallback {
                    size_name: size.id.0.clone(),
                    size: size_view(size.size),
                    refused: view.state.refused_sizes.contains(&size.id.0),
                },
                None => BoatLabelFit::NoSizeFits,
            }
        }
        _ => BoatLabelFit::Unavailable,
    };
    let boat_fits = matches!(
        fit.boat,
        BoatLabelFit::Fitting { .. } | BoatLabelFit::LargestFallback { .. }
    );
    let boat_known =
        fit.boat_connected == Some(false) || !matches!(fit.boat, BoatLabelFit::Unavailable);
    fit.placement = match (own_fits, own_unknown, boat_fits, boat_known) {
        (true, _, true, _) => LabelPlacement::MachinesThenBoat,
        (true, _, false, true) => LabelPlacement::MachinesOnly,
        (false, false, true, _) => LabelPlacement::BoatOnly,
        (false, false, false, true) => LabelPlacement::Waits,
        _ => LabelPlacement::Unavailable,
    };
}

fn size_view(size: VmSize) -> JobVmSize {
    JobVmSize {
        vcpus: size.vcpu,
        memory_bytes: size.memory_bytes.to_string(),
        disk_bytes: size.disk_bytes.to_string(),
    }
}
