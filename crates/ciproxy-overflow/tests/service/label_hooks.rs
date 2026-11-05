//! Current label changes during deterministic provider operations.

use std::sync::{Arc, Mutex};

use ciproxy_store_interface::store::Store;

use super::fixtures::id;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum LabelStage {
    Account,
    Capacity,
    Create,
    Recover,
    Inspect,
    Inventory,
    Bootstrap,
    Prepare,
    Input,
    Launch,
    LaunchInspection,
    Usage,
    Destroy,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Change {
    BoatOff,
    Cpu,
    Memory,
    Disk,
}

struct Pending {
    stage: LabelStage,
    change: Change,
    store: Arc<dyn Store>,
}

#[derive(Clone, Default)]
pub(super) struct LabelHook {
    pending: Arc<Mutex<Option<Pending>>>,
    pub(super) applied: Arc<Mutex<usize>>,
}

impl LabelHook {
    pub(super) fn set(&self, stage: LabelStage, change: Change, store: Arc<dyn Store>) {
        *self.pending.lock().unwrap() = Some(Pending {
            stage,
            change,
            store,
        });
    }

    pub(super) fn apply(&self, stage: LabelStage) {
        let change = {
            let mut pending = self.pending.lock().unwrap();
            if pending
                .as_ref()
                .is_some_and(|pending| pending.stage == stage)
            {
                pending.take()
            } else {
                None
            }
        };
        if let Some(pending) = change {
            change_label(&pending.store, pending.change);
            *self.applied.lock().unwrap() += 1;
        }
    }
}

pub(super) fn change_label(store: &Arc<dyn Store>, change: Change) {
    let mut label = store.labels(id(1)).unwrap().remove(0);
    match change {
        Change::BoatOff => label.allow_boat = false,
        Change::Cpu => label.size.vcpus += 2,
        Change::Memory => label.size.memory_gb += 4,
        Change::Disk => label.size.disk_gb += 20,
    }
    store.update_label(id(1), &label).unwrap();
}
