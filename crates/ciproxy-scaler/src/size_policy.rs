//! One size policy for new Boat starts and existing idle Boat runners.

use ciproxy_protocol::resources::VmSize;

use crate::boat::{BoatAccount, BoatSize, BoatSizeId, Refusal};
use crate::capacity::eligible;
use crate::snapshot::{Label, NothingFits, Snapshot};

/// Resource eligibility without account spend or start-availability policy.
pub(crate) struct SizePolicy<'a> {
    /// Whether a connected, non-draining allowed own machine fits the size.
    pub(crate) own_fits: bool,
    /// Cheapest fitting catalog size or the current largest-size exception.
    pub(crate) boat: Option<BoatChoice<'a>>,
    requested: VmSize,
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
pub fn select_boat_size<'a>(
    sizes: impl Iterator<Item = &'a BoatSize> + Clone,
    largest_size: Option<&BoatSizeId>,
    requested: VmSize,
    own_fits: bool,
    nothing_fits: NothingFits,
) -> Option<BoatChoice<'a>> {
    let fitting = sizes
        .clone()
        .filter(|size| covers(size.size, requested))
        .min_by_key(|size| size.price);
    if let Some(size) = fitting {
        Some(BoatChoice::Fitting(size))
    } else if !own_fits && nothing_fits == NothingFits::LargestBoatSize {
        sizes
            .into_iter()
            .find(|size| Some(&size.id) == largest_size)
            .map(BoatChoice::Largest)
    } else {
        None
    }
}

impl<'a> SizePolicy<'a> {
    pub(crate) fn for_label(snapshot: &'a Snapshot, label: &Label) -> Self {
        let own_fits = snapshot
            .machines
            .iter()
            .any(|machine| eligible(machine, label));
        let boat = select_boat_size(
            snapshot
                .boat
                .sizes
                .iter()
                .filter(|size| allowed(&snapshot.boat, size)),
            snapshot.boat.largest_size.as_ref(),
            label.size,
            own_fits,
            label.nothing_fits,
        );
        Self {
            own_fits,
            boat,
            requested: label.size,
        }
    }

    /// A fitting VM stays usable. An undersized VM needs the current exception.
    pub(crate) fn allows_boat_runner(&self, actual: VmSize) -> bool {
        covers(actual, self.requested)
            || matches!(&self.boat, Some(BoatChoice::Largest(size)) if size.size == actual)
    }
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
