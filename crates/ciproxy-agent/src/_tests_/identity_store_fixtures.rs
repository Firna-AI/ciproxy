//! Captured state and file effects for identity lifecycle regressions.

use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use crate::application_setup::LoadedInvocation;
use crate::boundaries::{ServiceMock, StoreMock};
use crate::endpoint::{AgentEndpoint, EndpointFilesMock, EndpointSettingsMock};
use crate::identity_store::IdentityStore;
use crate::runtime_ownership::{RuntimeClaimsMock, RuntimeOwner};
use crate::state::MachineState;

pub(super) fn legacy_state() -> MachineState {
    serde_json::from_str(
        r#"{
        "identity":{"workspace_id":"workspace-1","machine_id":"machine-1","credential":"synthetic-machine-secret"},
        "mode":"Draining","runners":[]
    }"#,
    )
    .unwrap()
}

pub(super) struct IdentityFiles {
    pub(super) mock: Arc<Unimock>,
    pub(super) state: Arc<Mutex<MachineState>>,
    pub(super) legacy: Arc<Mutex<Option<String>>>,
    pub(super) effects: Arc<Mutex<Vec<&'static str>>>,
}

impl IdentityFiles {
    pub(super) fn new(initial: MachineState) -> Self {
        let state = Arc::new(Mutex::new(initial));
        let legacy = Arc::new(Mutex::new(Some(
            "wss://preview.ciproxy.example/agent/".to_owned(),
        )));
        let effects = Arc::new(Mutex::new(Vec::new()));
        let loads = state.clone();
        let saves = state.clone();
        let clears = state.clone();
        let reads = legacy.clone();
        let removes = legacy.clone();
        let writes = effects.clone();
        let removals = effects.clone();
        let stops = effects.clone();
        let clearing = effects.clone();
        let mock = Arc::new(
            Unimock::new((
                StoreMock::load
                    .each_call(matching!())
                    .answers_arc(Arc::new(move |_| Ok(loads.lock().unwrap().clone()))),
                StoreMock::save
                    .each_call(matching!(_))
                    .answers_arc(Arc::new(move |_, state| {
                        writes.lock().unwrap().push("save");
                        *saves.lock().unwrap() = state.clone();
                        Ok(())
                    })),
                StoreMock::clear
                    .each_call(matching!())
                    .answers_arc(Arc::new(move |_| {
                        clearing.lock().unwrap().push("clear");
                        *clears.lock().unwrap() = MachineState::default();
                        Ok(())
                    })),
                EndpointFilesMock::read
                    .each_call(matching!())
                    .answers_arc(Arc::new(move |_| Ok(reads.lock().unwrap().clone()))),
                EndpointFilesMock::remove
                    .each_call(matching!())
                    .answers_arc(Arc::new(move |_| {
                        removals.lock().unwrap().push("remove");
                        *removes.lock().unwrap() = None;
                        Ok(())
                    })),
                EndpointSettingsMock::resolve
                    .each_call(matching!(_))
                    .answers(&|_, _| Ok(AgentEndpoint::default())),
                ServiceMock::stop
                    .each_call(matching!())
                    .answers_arc(Arc::new(move |_| {
                        stops.lock().unwrap().push("stop");
                        Ok(())
                    })),
                RuntimeClaimsMock::claim
                    .each_call(matching!())
                    .answers(&|_| Ok(Arc::new(Unimock::new(())) as Arc<dyn RuntimeOwner>)),
            ))
            .no_verify_in_drop(),
        );
        Self {
            mock,
            state,
            legacy,
            effects,
        }
    }

    pub(super) fn store(&self) -> Arc<IdentityStore> {
        Arc::new(IdentityStore {
            store: self.mock.clone(),
            endpoints: self.mock.clone(),
        })
    }

    pub(super) fn setup(&self) -> LoadedInvocation {
        LoadedInvocation {
            endpoints: self.mock.clone(),
            service: self.mock.clone(),
            runtime: self.mock.clone(),
            identity: self.store(),
        }
    }
}
