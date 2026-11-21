//! Choose exact free logical slots without imposing a contiguous range.

use std::collections::BTreeSet;

use ciproxy_protocol::resources::{Slots, VmSize};
use ciproxy_store_interface::own_runners::OwnRunner;
use ciproxy_store_interface::runners::{RunnerPlace, RunnerState};
use uuid::Uuid;

pub(crate) fn allocate_slots(
    machine_id: Uuid,
    requested: VmSize,
    capacity: u64,
    runners: &[OwnRunner],
) -> Option<Vec<u64>> {
    let mut used = BTreeSet::new();
    for intent in runners
        .iter()
        .filter(|intent| intent.runner.state != RunnerState::Stopped)
    {
        if let RunnerPlace::Machine {
            machine_id: owner,
            first_slot: _,
        } = intent.runner.place
            && owner == machine_id
        {
            used.extend(intent.slot_indices.iter().copied());
        }
    }
    let required = Slots::for_vm(requested).0 as usize;
    let indices: Vec<_> = (0..capacity)
        .filter(|slot| !used.contains(slot))
        .take(required)
        .collect();
    (indices.len() == required).then_some(indices)
}
