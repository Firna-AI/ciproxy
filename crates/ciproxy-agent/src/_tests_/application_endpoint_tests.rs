//! Uninstall must never pair saved credentials with an invented default target.

use unimock::{MockFn, Unimock, matching};

use crate::application_endpoint::configure;
use crate::cli::Action;
use crate::endpoint::EndpointSettingsMock;
use crate::error::Error;

#[test]
fn uninstall_preserves_the_saved_preview_endpoint_for_metadata() {
    let settings = Unimock::new(
        EndpointSettingsMock::saved
            .some_call(matching!())
            .returns(Ok(Some(
                "wss://preview.ciproxy.example:8443/agent/".parse().unwrap(),
            ))),
    );
    let action = configure(Action::Uninstall, &settings).unwrap();
    assert_eq!(
        action.endpoint().map(|endpoint| endpoint.as_str()),
        Some("wss://preview.ciproxy.example:8443/agent/")
    );
}

#[test]
fn uninstall_without_a_trusted_saved_endpoint_keeps_local_cleanup_available() {
    for saved in [Ok(None), Err(Error::AgentEndpoint)] {
        let settings = Unimock::new(
            EndpointSettingsMock::saved
                .some_call(matching!())
                .returns(saved),
        );
        let action = configure(Action::Uninstall, &settings).unwrap();
        assert_eq!(action.endpoint().map(|endpoint| endpoint.as_str()), None);
    }
}
