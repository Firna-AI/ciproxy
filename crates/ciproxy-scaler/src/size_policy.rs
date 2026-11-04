//! One size policy for new Boat starts and existing idle Boat runners.

use ciproxy_protocol::resources::VmSize;

use crate::boat::{BoatAccount, BoatSize, Refusal};
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
pub(crate) enum BoatChoice<'a> {
    /// The size covers every requested resource dimension.
    Fitting(&'a BoatSize),
    /// Neither destination fits and the label permits the largest Boat size.
    Largest(&'a BoatSize),
}

impl<'a> SizePolicy<'a> {
    pub(crate) fn for_label(snapshot: &'a Snapshot, label: &Label) -> Self {
        let own_fits = snapshot
            .machines
            .iter()
            .any(|machine| eligible(machine, label));
        let fitting = snapshot
            .boat
            .sizes
            .iter()
            .filter(|size| allowed(&snapshot.boat, size) && covers(size.size, label.size))
            .min_by_key(|size| size.price);
        let boat = if let Some(size) = fitting {
            Some(BoatChoice::Fitting(size))
        } else if !own_fits && label.nothing_fits == NothingFits::LargestBoatSize {
            snapshot
                .boat
                .sizes
                .iter()
                .find(|size| {
                    Some(&size.id) == snapshot.boat.largest_size.as_ref()
                        && allowed(&snapshot.boat, size)
                })
                .map(BoatChoice::Largest)
        } else {
            None
        };
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
