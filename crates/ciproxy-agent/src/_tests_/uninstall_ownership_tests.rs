//! Uninstall must own the runtime before identity and empty-area cleanup.

use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use crate::application_setup::{InvocationSetup, LoadedInvocation};
use crate::area::AreaFilesMock;
use crate::boundaries::{ServiceMock, StoreMock};
use crate::cli::Action;
use crate::controller::AgentMock;
use crate::host_fixtures::inventory;
use crate::host_preparation::{FinalUninstall, Uninstall, UninstallState};
use crate::identity_store::IdentityStore;
use crate::runtime_ownership::{RuntimeClaimsMock, RuntimeOwner};
use crate::state::MachineState;

use super::run_ownership_tests::{files, legacy};

#[tokio::test]
async fn uninstall_waits_before_identity_migration_and_holds_ownership_through_empty_cleanup() {
    let state = Arc::new(Mutex::new(legacy()));
    let endpoint = Arc::new(Mutex::new(Some(
        "wss://preview.ciproxy.example/agent/".to_owned(),
    )));
    let events = Arc::new(Mutex::new(Vec::new()));
    let owner: Arc<dyn RuntimeOwner> = Arc::new(Unimock::new(()));
    let held = Arc::downgrade(&owner);
    {
        let retained = {
            let stops = events.clone();
            let claims = events.clone();
            let mock = Arc::new(Unimock::new((
                ServiceMock::stop
                    .some_call(matching!())
                    .answers_arc(Arc::new(move |_| {
                        stops.lock().unwrap().push("stop");
                        Ok(())
                    })),
                RuntimeClaimsMock::wait
                    .some_call(matching!())
                    .answers_arc(Arc::new(move |_| {
                        claims.lock().unwrap().push("wait");
                        Ok(owner.clone())
                    })),
            )));
            let files = files(state.clone(), endpoint, events.clone());
            let setup = LoadedInvocation {
                endpoints: Arc::new(Unimock::new(())),
                service: mock.clone(),
                runtime: mock,
                identity: Arc::new(IdentityStore {
                    store: files.clone(),
                    endpoints: files,
                }),
            };
            setup
                .prepare(Action::Uninstall)
                .unwrap()
                .ownership
                .unwrap()
                .runtime()
        };
        assert_eq!(
            *events.lock().unwrap(),
            vec!["stop", "wait", "load", "read", "save", "read"]
        );
        assert_eq!(held.strong_count(), 1);
        let during_cleanup = held.clone();
        let mock = Arc::new(Unimock::new((
            AreaFilesMock::inspect
                .some_call(matching!())
                .returns(Ok(inventory())),
            StoreMock::load
                .some_call(matching!())
                .returns(Ok(MachineState::default())),
            AgentMock::remove.some_call(matching!()).returns(Ok(())),
            AgentMock::complete_removal
                .some_call(matching!(_))
                .answers_arc(Arc::new(move |_, _| {
                    assert_eq!(
                        during_cleanup.strong_count(),
                        1,
                        "empty cleanup must keep runtime ownership"
                    );
                    Ok(true)
                })),
        )));
        let uninstall = FinalUninstall {
            _runtime: retained,
            files: mock.clone(),
            images: mock.clone(),
            preparation: mock.clone(),
            service: mock.clone(),
            store: mock.clone(),
            agent: mock,
        };
        assert_eq!(uninstall.start().await.unwrap(), UninstallState::Complete);
        assert_eq!(held.strong_count(), 1);
    }
    assert_eq!(held.strong_count(), 0);
}
