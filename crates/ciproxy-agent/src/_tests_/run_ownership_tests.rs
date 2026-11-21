//! Run must own the existing runtime before it can persist recovered identity.

use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use crate::application_setup::{InvocationError, InvocationSetup, LoadedInvocation};
use crate::boundaries::StoreMock;
use crate::cli::Action;
use crate::endpoint::EndpointFilesMock;
use crate::error::Error;
use crate::fixtures::identity;
use crate::identity_store::IdentityStore;
use crate::runtime_ownership::{RuntimeClaimsMock, RuntimeOwner};
use crate::state::MachineState;

pub(super) fn legacy() -> MachineState {
    let mut identity = identity();
    identity.endpoint = None;
    MachineState {
        identity: Some(identity),
        ..MachineState::default()
    }
}

pub(super) fn files(
    state: Arc<Mutex<MachineState>>,
    endpoint: Arc<Mutex<Option<String>>>,
    events: Arc<Mutex<Vec<&'static str>>>,
) -> Arc<Unimock> {
    let load = state.clone();
    let save = state;
    let read = endpoint.clone();
    let remove = endpoint;
    let loads = events.clone();
    let writes = events.clone();
    let reads = events.clone();
    Arc::new(
        Unimock::new((
            StoreMock::load
                .each_call(matching!())
                .answers_arc(Arc::new(move |_| {
                    loads.lock().unwrap().push("load");
                    Ok(load.lock().unwrap().clone())
                })),
            StoreMock::save
                .each_call(matching!(_))
                .answers_arc(Arc::new(move |_, state| {
                    writes.lock().unwrap().push("save");
                    *save.lock().unwrap() = state.clone();
                    Ok(())
                })),
            EndpointFilesMock::read
                .each_call(matching!())
                .answers_arc(Arc::new(move |_| {
                    reads.lock().unwrap().push("read");
                    Ok(read.lock().unwrap().clone())
                })),
            EndpointFilesMock::remove
                .each_call(matching!())
                .answers_arc(Arc::new(move |_| {
                    events.lock().unwrap().push("remove");
                    *remove.lock().unwrap() = None;
                    Ok(())
                })),
        ))
        .no_verify_in_drop(),
    )
}

#[test]
fn busy_run_preserves_legacy_state_without_loading_saving_or_removing_it() {
    let initial = legacy();
    let state = Arc::new(Mutex::new(initial.clone()));
    let endpoint = Arc::new(Mutex::new(Some(
        "wss://preview.ciproxy.example/agent/".to_owned(),
    )));
    let events = Arc::new(Mutex::new(Vec::new()));
    let claims = events.clone();
    let runtime = Arc::new(Unimock::new(
        RuntimeClaimsMock::claim
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                claims.lock().unwrap().push("claim");
                Err(Error::ImageOwnership)
            })),
    ));
    let files = files(state.clone(), endpoint.clone(), events.clone());
    let setup = LoadedInvocation {
        endpoints: Arc::new(Unimock::new(())),
        service: Arc::new(Unimock::new(())),
        runtime,
        identity: Arc::new(IdentityStore {
            store: files.clone(),
            endpoints: files,
        }),
    };
    let result = setup.prepare(Action::Run);
    assert!(matches!(
        result,
        Err(InvocationError::Setup(Error::ImageOwnership))
    ));
    assert_eq!(*events.lock().unwrap(), vec!["claim"]);
    assert_eq!(*state.lock().unwrap(), initial);
    assert_eq!(
        endpoint.lock().unwrap().as_deref(),
        Some("wss://preview.ciproxy.example/agent/")
    );
}

#[test]
fn run_claim_precedes_migration_and_stays_owned_after_setup_handoff() {
    let state = Arc::new(Mutex::new(legacy()));
    let endpoint = Arc::new(Mutex::new(Some(
        "wss://preview.ciproxy.example/agent/".to_owned(),
    )));
    let events = Arc::new(Mutex::new(Vec::new()));
    let claims = events.clone();
    let owner: Arc<dyn RuntimeOwner> = Arc::new(Unimock::new(()));
    let held = Arc::downgrade(&owner);
    {
        let retained = {
            let runtime = Arc::new(Unimock::new(
                RuntimeClaimsMock::claim
                    .some_call(matching!())
                    .answers_arc(Arc::new(move |_| {
                        claims.lock().unwrap().push("claim");
                        Ok(owner.clone())
                    })),
            ));
            let files = files(state.clone(), endpoint.clone(), events.clone());
            let setup = LoadedInvocation {
                endpoints: Arc::new(Unimock::new(())),
                service: Arc::new(Unimock::new(())),
                runtime,
                identity: Arc::new(IdentityStore {
                    store: files.clone(),
                    endpoints: files,
                }),
            };
            let prepared = setup.prepare(Action::Run).unwrap();
            prepared.ownership.unwrap().runtime()
        };
        assert_eq!(held.strong_count(), 1);
        assert_eq!(
            *events.lock().unwrap(),
            vec!["claim", "load", "read", "save", "read"]
        );
        assert_eq!(
            state
                .lock()
                .unwrap()
                .identity
                .as_ref()
                .and_then(|identity| identity.endpoint.as_ref())
                .map(|endpoint| endpoint.as_str()),
            Some("wss://preview.ciproxy.example/agent/")
        );
        assert_eq!(
            endpoint.lock().unwrap().as_deref(),
            Some("wss://preview.ciproxy.example/agent/")
        );
        let _runtime_lifetime = retained;
    }
    assert_eq!(held.strong_count(), 0);
}
