//! Resource fit for new starts and existing idle provider runners.

use ciproxy_protocol::resources::VmSize;

use crate::capacity::own_fits;
use crate::provider::{
    ProviderAccount, ProviderId, ProviderRunner, ProviderSize, ProviderSizeId, Refusal,
};
use crate::snapshot::{Label, NothingFits, Place, Snapshot};

/// A catalog choice under the label's current resource policy.
pub enum ProviderChoice<'a> {
    /// The size covers every requested resource dimension.
    Fitting(&'a ProviderSize),
    /// No place fits and the label permits this provider's largest size.
    Largest(&'a ProviderSize),
}

/// Choose the cheapest fitting size or the label's largest-size fallback.
/// Supply only eligible sizes after applying current provider size refusals.
/// The required size includes the complete allocated root.
pub fn select_provider_size<'a>(
    sizes: impl Iterator<Item = &'a ProviderSize> + Clone,
    largest_size: Option<&ProviderSizeId>,
    required: VmSize,
    image_root_bytes: u64,
    another_place_fits: bool,
    nothing_fits: NothingFits,
) -> Option<ProviderChoice<'a>> {
    if let Some(size) = fitting_size(sizes.clone(), required) {
        Some(ProviderChoice::Fitting(size))
    } else if !another_place_fits && nothing_fits == NothingFits::LargestProviderSize {
        largest_for_image(sizes, largest_size, image_root_bytes).map(ProviderChoice::Largest)
    } else {
        None
    }
}

impl ProviderAccount {
    pub(crate) fn fitting_size(&self, requested: VmSize) -> Option<&ProviderSize> {
        let required = self.connected_geometry()?.vm_size(requested)?;
        fitting_size(self.eligible_sizes(), required)
    }

    pub(crate) fn largest_for_start(&self, requested: VmSize) -> Option<&ProviderSize> {
        let geometry = self.connected_geometry()?;
        geometry.vm_size(requested)?;
        largest_for_image(
            self.eligible_sizes(),
            self.facts.largest_size.as_ref(),
            geometry.image_root_bytes,
        )
    }

    fn eligible_sizes(&self) -> impl Iterator<Item = &ProviderSize> {
        self.facts.sizes.iter().filter(|size| {
            !self
                .facts
                .refusals
                .iter()
                .any(|refusal| matches!(refusal, Refusal::SizeNotOnPlan(id) if *id == size.id))
        })
    }
}

/// Test physical fit for a start at one place of the label order.
pub(crate) fn place_fits(snapshot: &Snapshot, label: &Label, place: Place) -> bool {
    match place {
        Place::OwnMachines => own_fits(snapshot, label),
        Place::Provider(id) => snapshot
            .provider(id)
            .is_some_and(|provider| provider.fitting_size(label.size).is_some()),
    }
}

/// Retain saved fit or the current exception without a new-image floor.
pub(crate) fn allows_provider_runner(
    snapshot: &Snapshot,
    label: &Label,
    provider_id: ProviderId,
    retained: &ProviderRunner,
) -> bool {
    let required = VmSize {
        disk_bytes: retained.required_root_disk_bytes,
        ..retained.runner.requested_size
    };
    if covers(retained.runner.size, required) {
        return true;
    }
    if label.nothing_fits != NothingFits::LargestProviderSize
        || label
            .places()
            .any(|place| place_fits(snapshot, label, place))
    {
        return false;
    }
    let Some(provider) = snapshot.provider(provider_id) else {
        return false;
    };
    provider.eligible_sizes().any(|size| {
        Some(&size.id) == provider.facts.largest_size.as_ref() && size.size == retained.runner.size
    })
}

fn fitting_size<'a>(
    sizes: impl Iterator<Item = &'a ProviderSize>,
    required: VmSize,
) -> Option<&'a ProviderSize> {
    sizes
        .filter(|size| covers(size.size, required))
        .min_by_key(|size| size.price)
}

fn largest_for_image<'a>(
    mut sizes: impl Iterator<Item = &'a ProviderSize>,
    id: Option<&ProviderSizeId>,
    image_root_bytes: u64,
) -> Option<&'a ProviderSize> {
    let id = id?;
    sizes.find(|size| &size.id == id && size.size.disk_bytes >= image_root_bytes)
}

fn covers(actual: VmSize, requested: VmSize) -> bool {
    actual.vcpu >= requested.vcpu
        && actual.memory_bytes >= requested.memory_bytes
        && actual.disk_bytes >= requested.disk_bytes
}
