//! A bad install command leaves the installed service and identity untouched.

use std::sync::{Arc, Mutex};

use ciproxy_protocol::secret::Secret;
use unimock::{MockFn, Unimock, matching};

use crate::application_setup::{InvocationError, InvocationSetup, LoadedInvocation};
use crate::boundaries::{ServiceMock, StoreMock};
use crate::cli::Action;
use crate::endpoint::{CommandEndpoint, EndpointFilesMock};
use crate::error::Error;
use crate::state::MachineState;

#[test]
fn invalid_install_endpoint_does_not_stop_the_service_or_load_state() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let resolves = events.clone();
    let stops = events.clone();
    let loads = events.clone();
    let files = Arc::new(Unimock::new(
        EndpointFilesMock::environment
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                resolves.lock().unwrap().push("resolve");
                Ok(Some("https://invalid.ciproxy.example/agent/".to_owned()))
            })),
    ));
    let dependencies = Arc::new(
        Unimock::new((
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
                    Ok(MachineState::default())
                })),
        ))
        .no_verify_in_drop(),
    );
    let setup = LoadedInvocation {
        endpoints: Arc::new(CommandEndpoint { files }),
        service: dependencies.clone(),
        store: dependencies,
    };
    let result = setup.prepare(Action::Install {
        token: Secret::new("synthetic-install-token"),
        agent_url: None,
    });
    assert!(matches!(
        result,
        Err(InvocationError::Setup(Error::AgentEndpoint))
    ));
    assert_eq!(*events.lock().unwrap(), vec!["resolve"]);
}
