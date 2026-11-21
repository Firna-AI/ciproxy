//! One size policy for new Boat starts and existing idle Boat runners.

use ciproxy_protocol::resources::VmSize;

use crate::boat::{BoatAccount, BoatRunner, BoatSize, BoatSizeId, Refusal};
use crate::capacity::eligible;
use crate::snapshot::{Label, NothingFits, Snapshot};

/// Resource eligibility without account spend or start-availability policy.
pub(crate) struct SizePolicy<'a> {
    /// Whether a connected, non-draining allowed own machine fits the size.
    pub(crate) own_fits: bool,
    /// Cheapest fitting catalog size or the current largest-size exception.
    pub(crate) boat: Option<BoatChoice<'a>>,
}

/// A catalog choice under the label's current resource policy.
pub enum BoatChoice<'a> {
    /// The size covers every requested resource dimension.
    Fitting(&'a BoatSize),
    /// Neither destination fits and the label permits the largest Boat size.
    Largest(&'a BoatSize),
}

/// Choose the cheapest fitting size or the label's current largest-size exception.
/// Supply only eligible sizes after applying current provider size refusals.
/// `required` contains the complete allocated root, rather than label free bytes.
pub fn select_boat_size<'a>(
    sizes: impl Iterator<Item = &'a BoatSize> + Clone,
    largest_size: Option<&BoatSizeId>,
    required: VmSize,
    image_root_bytes: u64,
    own_fits: bool,
    nothing_fits: NothingFits,
) -> Option<BoatChoice<'a>> {
    let fitting = sizes
        .clone()
        .filter(|size| covers(size.size, required))
        .min_by_key(|size| size.price);
    if let Some(size) = fitting {
        Some(BoatChoice::Fitting(size))
    } else if !own_fits && nothing_fits == NothingFits::LargestBoatSize {
        sizes
            .into_iter()
            .find(|size| Some(&size.id) == largest_size && size.size.disk_bytes >= image_root_bytes)
            .map(BoatChoice::Largest)
    } else {
        None
    }
}

impl<'a> SizePolicy<'a> {
    pub(crate) fn for_label(snapshot: &'a Snapshot, label: &Label) -> Self {
        let own_fits = own_fits(snapshot, label);
        let required = snapshot.root_sizing.vm_size(label.size);
        let boat = required.and_then(|required| {
            select_boat_size(
                snapshot
                    .boat
                    .sizes
                    .iter()
                    .filter(|size| allowed(&snapshot.boat, size)),
                snapshot.boat.largest_size.as_ref(),
                required,
                snapshot.root_sizing.image_root_bytes,
                own_fits,
                label.nothing_fits,
            )
        });
        Self { own_fits, boat }
    }
}

/// Retain saved fit or the current exception without a new-image floor.
pub(crate) fn allows_boat_runner(
    snapshot: &Snapshot,
    label: &Label,
    retained: &BoatRunner,
) -> bool {
    let required = VmSize {
        disk_bytes: retained.required_root_disk_bytes,
        ..retained.runner.requested_size
    };
    if covers(retained.runner.size, required) {
        return true;
    }
    if label.nothing_fits != NothingFits::LargestBoatSize || own_fits(snapshot, label) {
        return false;
    }
    let mut sizes = snapshot
        .boat
        .sizes
        .iter()
        .filter(|size| allowed(&snapshot.boat, size));
    let fitting = snapshot
        .root_sizing
        .vm_size(label.size)
        .is_some_and(|required| sizes.clone().any(|size| covers(size.size, required)));
    !fitting
        && sizes.any(|size| {
            Some(&size.id) == snapshot.boat.largest_size.as_ref()
                && size.size == retained.runner.size
        })
}

fn own_fits(snapshot: &Snapshot, label: &Label) -> bool {
    snapshot
        .machines
        .iter()
        .any(|machine| eligible(machine, label, snapshot.root_sizing))
}

fn allowed(boat: &BoatAccount, size: &BoatSize) -> bool {
    !boat
        .refusals
        .iter()
        .any(|refusal| matches!(refusal, Refusal::SizeNotOnPlan(id) if *id == size.id))
}

fn covers(actual: VmSize, requested: VmSize) -> bool {
    actual.vcpu >= requested.vcpu
        && actual.memory_bytes >= requested.memory_bytes
        && actual.disk_bytes >= requested.disk_bytes
}
