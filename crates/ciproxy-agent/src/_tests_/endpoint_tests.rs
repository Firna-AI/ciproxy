//! Install command settings never select a route from retained machine state.

use std::sync::Arc;

use unimock::{MockFn, Unimock, matching};

use crate::endpoint::{CommandEndpoint, EndpointFilesMock, EndpointSettings};
use crate::error::Error;

#[test]
fn explicit_endpoint_takes_priority_over_environment() {
    let settings = CommandEndpoint {
        files: Arc::new(Unimock::new(())),
    };
    let endpoint = "wss://preview.ciproxy.example/agent/".parse().unwrap();
    assert_eq!(
        settings.resolve(Some(&endpoint)).unwrap().as_str(),
        "wss://preview.ciproxy.example/agent/"
    );
}

#[test]
fn environment_endpoint_configures_registration() {
    let settings = CommandEndpoint {
        files: Arc::new(Unimock::new(
            EndpointFilesMock::environment
                .some_call(matching!())
                .returns(Ok(Some("wss://preview.ciproxy.example/agent/".to_owned()))),
        )),
    };
    assert_eq!(
        settings.resolve(None).unwrap().as_str(),
        "wss://preview.ciproxy.example/agent/"
    );
}

#[test]
fn environment_endpoint_requires_wss() {
    let settings = CommandEndpoint {
        files: Arc::new(Unimock::new(
            EndpointFilesMock::environment
                .some_call(matching!())
                .returns(Ok(Some(
                    "https://preview.ciproxy.example/agent/".to_owned(),
                ))),
        )),
    };
    assert!(matches!(settings.resolve(None), Err(Error::AgentEndpoint)));
}

#[test]
fn fresh_machine_uses_the_production_endpoint() {
    let settings = CommandEndpoint {
        files: Arc::new(Unimock::new(
            EndpointFilesMock::environment
                .some_call(matching!())
                .returns(Ok(None)),
        )),
    };
    assert_eq!(
        settings.resolve(None).unwrap().as_str(),
        "wss://app.ciproxy.dev/agent/"
    );
}
