//! Legacy endpoint migration commits the full identity before retiring the file.

use std::sync::{Arc, Mutex};

use ciproxy_protocol::secret::Secret;
use unimock::{MockFn, Unimock, matching};

use crate::application_endpoint::ConfiguredAction;
use crate::application_setup::{InvocationSetup, LoadedInvocation, PreparedInvocation};
use crate::boundaries::{ServiceMock, StoreMock};
use crate::cli::Action;
use crate::endpoint::{AgentEndpoint, EndpointFilesMock, EndpointSettingsMock};
use crate::identity_store::IdentityStore;
use crate::runtime_ownership::RuntimeClaimsMock;
use crate::state::MachineState;

#[test]
fn install_stops_the_service_before_migrating_the_legacy_endpoint_once() {
    let state: MachineState = serde_json::from_str(r#"{
        "identity":{"workspace_id":"workspace-1","machine_id":"machine-1","credential":"machine-secret"},
        "mode":"Active","runners":[]
    }"#).unwrap();
    let stored = Arc::new(Mutex::new(state));
    let load = stored.clone();
    let save = stored.clone();
    let observed = stored.clone();
    let events = Arc::new(Mutex::new(Vec::new()));
    let writes = events.clone();
    let removals = events.clone();
    let stops = events.clone();
    let resolves = events.clone();
    let loads = events.clone();
    let claims = events.clone();
    let mock = Arc::new(Unimock::new((
        RuntimeClaimsMock::claim
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                claims.lock().unwrap().push("claim");
                Ok(Arc::new(Unimock::new(())))
            })),
        EndpointSettingsMock::resolve
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, _| {
                resolves.lock().unwrap().push("resolve");
                Ok(AgentEndpoint::default())
            })),
        ServiceMock::stop
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| {
                stops.lock().unwrap().push("stop");
                Ok(())
            })),
        StoreMock::load
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| {
                loads.lock().unwrap().push("load");
                Ok(load.lock().unwrap().clone())
            })),
        StoreMock::save
            .some_call(matching!(_))
            .answers_arc(Arc::new(move |_, state| {
                writes.lock().unwrap().push("state");
                *save.lock().unwrap() = state.clone();
                Ok(())
            })),
        EndpointFilesMock::read
            .some_call(matching!())
            .returns(Ok(Some("wss://preview.ciproxy.example/agent/".to_owned()))),
        EndpointFilesMock::remove
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| {
                assert_eq!(
                    observed
                        .lock()
                        .unwrap()
                        .identity
                        .as_ref()
                        .unwrap()
                        .endpoint
                        .as_ref()
                        .unwrap()
                        .as_str(),
                    "wss://preview.ciproxy.example/agent/"
                );
                removals.lock().unwrap().push("remove");
                Ok(())
            })),
    )));
    let setup = LoadedInvocation {
        endpoints: mock.clone(),
        service: mock.clone(),
        runtime: mock.clone(),
        identity: Arc::new(IdentityStore {
            store: mock.clone(),
            endpoints: mock,
        }),
    };
    let PreparedInvocation {
        action,
        state: first,
        ..
    } = setup
        .prepare(Action::Install {
            token: Secret::new("synthetic-install-token"),
            agent_url: None,
        })
        .unwrap();
    let ConfiguredAction::Install { endpoint, .. } = action else {
        panic!("install must retain its selected command endpoint");
    };
    assert_eq!(endpoint.as_str(), "wss://app.ciproxy.dev/agent/");
    assert_eq!(
        first
            .identity
            .as_ref()
            .and_then(|identity| identity.endpoint.as_ref())
            .map(|endpoint| endpoint.as_str()),
        Some("wss://preview.ciproxy.example/agent/")
    );
    let PreparedInvocation {
        action,
        state: second,
        ..
    } = setup.prepare(Action::Run).unwrap();
    assert_eq!(second, first);
    let ConfiguredAction::Run { endpoint } = action else {
        panic!("daemon startup must keep its action");
    };
    assert_eq!(
        endpoint.as_ref().map(|endpoint| endpoint.as_str()),
        Some("wss://preview.ciproxy.example/agent/")
    );
    assert_eq!(
        *events.lock().unwrap(),
        vec![
            "resolve", "stop", "load", "state", "remove", "claim", "load", "remove"
        ]
    );
    let encoded = serde_json::to_string(&*stored.lock().unwrap()).unwrap();
    assert!(encoded.contains(r#""endpoint":"wss://preview.ciproxy.example/agent/""#));
}
