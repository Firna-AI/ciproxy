//! Explicit install settings precede saved settings and the environment.

use std::sync::Arc;

use unimock::{MockFn, Unimock, matching};

use crate::endpoint::{EndpointFilesMock, EndpointSettings, SavedEndpoint};
use crate::error::Error;

#[test]
fn explicit_endpoint_takes_priority_over_environment_and_saved_settings() {
    let settings = SavedEndpoint {
        files: Arc::new(Unimock::new(())),
    };
    let endpoint = "wss://preview.ciproxy.example/agent/".parse().unwrap();
    assert_eq!(
        settings.resolve(Some(&endpoint)).unwrap().as_str(),
        "wss://preview.ciproxy.example/agent/"
    );
}

#[test]
fn environment_endpoint_configures_a_machine_without_saved_settings() {
    let settings = SavedEndpoint {
        files: Arc::new(Unimock::new((
            EndpointFilesMock::read
                .some_call(matching!())
                .returns(Ok(None)),
            EndpointFilesMock::environment
                .some_call(matching!())
                .returns(Ok(Some("wss://preview.ciproxy.example/agent/".to_owned()))),
        ))),
    };
    assert_eq!(
        settings.resolve(None).unwrap().as_str(),
        "wss://preview.ciproxy.example/agent/"
    );
}

#[test]
fn environment_endpoint_requires_wss() {
    let settings = SavedEndpoint {
        files: Arc::new(Unimock::new((
            EndpointFilesMock::read
                .some_call(matching!())
                .returns(Ok(None)),
            EndpointFilesMock::environment
                .some_call(matching!())
                .returns(Ok(Some(
                    "https://preview.ciproxy.example/agent/".to_owned(),
                ))),
        ))),
    };
    assert!(matches!(settings.resolve(None), Err(Error::AgentEndpoint)));
}

#[test]
fn saved_endpoint_takes_priority_over_inherited_environment() {
    let settings = SavedEndpoint {
        files: Arc::new(Unimock::new(
            EndpointFilesMock::read
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
fn fresh_machine_uses_the_production_endpoint() {
    let settings = SavedEndpoint {
        files: Arc::new(Unimock::new((
            EndpointFilesMock::environment
                .some_call(matching!())
                .returns(Ok(None)),
            EndpointFilesMock::read
                .some_call(matching!())
                .returns(Ok(None)),
        ))),
    };
    assert_eq!(
        settings.resolve(None).unwrap().as_str(),
        "wss://app.ciproxy.dev/agent/"
    );
}
