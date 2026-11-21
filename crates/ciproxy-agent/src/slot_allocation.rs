//! Stable logical slot sets for the lifetime of each VM.

use std::collections::BTreeSet;

use ciproxy_protocol::resources::{Slots, VmSize};

use crate::error::{Error, Result};
use crate::state::VmRecord;

pub(crate) fn allocations(runners: &[VmRecord]) -> Vec<Vec<u64>> {
    runners
        .iter()
        .map(|vm| vm.report.slot_indices.clone())
        .collect()
}

pub(crate) fn choose(
    size: VmSize,
    requested: &[u64],
    capacity: u64,
    runners: &[VmRecord],
) -> Result<Vec<u64>> {
    let used: BTreeSet<_> = allocations(runners).into_iter().flatten().collect();
    let required = Slots::for_vm(size).0 as usize;
    let slots: BTreeSet<_> = requested.iter().copied().collect();
    if slots.len() != required
        || slots
            .iter()
            .any(|slot| *slot >= capacity || used.contains(slot))
    {
        return Err(Error::Capacity);
    }
    Ok(slots.into_iter().collect())
}
