//! Independent organization writes at deterministic provider boundaries.

use std::sync::{Arc, Mutex};

use ciproxy_store_interface::organizations::AppState;
use ciproxy_store_interface::store::Store;

use super::fixtures::{at, id};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Stage {
    Created,
    Prepared,
    InputWritten,
}

struct Withdrawal {
    stage: Stage,
    state: AppState,
    store: Arc<dyn Store>,
}

#[derive(Clone, Default)]
pub(super) struct OrganizationHook(Arc<Mutex<Option<Withdrawal>>>);

impl OrganizationHook {
    pub(super) fn set(&self, stage: Stage, state: AppState, store: Arc<dyn Store>) {
        *self.0.lock().unwrap() = Some(Withdrawal {
            stage,
            state,
            store,
        });
    }

    pub(super) fn apply(&self, stage: Stage) {
        let withdrawal = {
            let mut pending = self.0.lock().unwrap();
            if pending
                .as_ref()
                .is_some_and(|withdrawal| withdrawal.stage == stage)
            {
                pending.take()
            } else {
                None
            }
        };
        if let Some(withdrawal) = withdrawal {
            withdraw(&withdrawal.store, withdrawal.state);
        }
    }
}

pub(super) fn withdraw(store: &Arc<dyn Store>, state: AppState) {
    match state {
        AppState::Suspended => store.set_app_suspended(id(1), id(3), true).unwrap(),
        AppState::Removed => store.record_app_removal(id(1), id(3), at()).unwrap(),
        AppState::Active => panic!("withdrawal needs a suspended or removed organization"),
    }
}
