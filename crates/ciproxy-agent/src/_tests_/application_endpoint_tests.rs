//! Retained credentials never use settings from another install command.

use std::sync::Arc;

use unimock::{MockFn, Unimock, matching};

use crate::application_endpoint::{ConfiguredAction, configure};
use crate::application_metadata::MetadataTarget;
use crate::boundaries::{Store, StoreMock};
use crate::cli::Action;
use crate::endpoint::EndpointFilesMock;
use crate::fixtures::identity;
use crate::identity_store::IdentityStore;
use crate::state::MachineState;

#[test]
fn daemon_and_uninstall_preserve_the_identity_preview_endpoint() {
    let mut identity = identity();
    identity.endpoint = Some("wss://preview.ciproxy.example:8443/agent/".parse().unwrap());
    let settings = Unimock::new(());
    let action = configure(Action::Run, &settings)
        .unwrap()
        .bind_identity(Some(&identity));
    let ConfiguredAction::Run { endpoint } = action else {
        panic!("run must keep the daemon action");
    };
    assert_eq!(
        endpoint.unwrap().as_str(),
        "wss://preview.ciproxy.example:8443/agent/"
    );
    let action = configure(Action::Uninstall, &settings).unwrap();
    assert!(matches!(action, ConfiguredAction::Uninstall));
    let target = MetadataTarget::for_machine(Some(&identity));
    let MetadataTarget::Hosted(source) = target else {
        panic!("uninstall must retain metadata for running jobs");
    };
    assert_eq!(
        source.url.expose_secret(),
        "https://preview.ciproxy.example:8443/agent/cache-metadata"
    );
}

#[test]
fn uninstall_without_a_trusted_identity_endpoint_keeps_local_cleanup_available() {
    for endpoint in [
        None,
        Some("https://invalid.ciproxy.example/agent/".to_owned()),
    ] {
        let mut identity = identity();
        identity.endpoint = None;
        let state = MachineState {
            identity: Some(identity),
            ..MachineState::default()
        };
        let mock = Arc::new(Unimock::new((
            StoreMock::load
                .some_call(matching!())
                .returns(Ok(state.clone())),
            EndpointFilesMock::read
                .some_call(matching!())
                .returns(Ok(endpoint)),
        )));
        let store = IdentityStore {
            store: mock.clone(),
            endpoints: mock.clone(),
        };
        let loaded = store.load().ok();
        assert_eq!(loaded, Some(state));
        let state = loaded.unwrap();
        let action = configure(Action::Uninstall, mock.as_ref()).unwrap();
        assert!(matches!(action, ConfiguredAction::Uninstall));
        let target = MetadataTarget::for_machine(state.identity.as_ref());
        assert!(matches!(target, MetadataTarget::Unavailable));
        let action = configure(Action::Run, mock.as_ref())
            .unwrap()
            .bind_identity(state.identity.as_ref());
        assert!(matches!(action, ConfiguredAction::Run { endpoint: None }));
    }
}
