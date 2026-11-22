//! Legacy endpoints remain available until their machine identity ends.

use std::sync::Arc;

use ciproxy_protocol::secret::Secret;
use unimock::{MockFn, Unimock, matching};

use crate::application_endpoint::ConfiguredAction;
use crate::application_setup::InvocationSetup;
use crate::boundaries::{ServiceMock, VmsMock};
use crate::cli::Action;
use crate::controller::Agent;
use crate::fixtures::controller;
use crate::identity_store_fixtures::{IdentityFiles, legacy_state};
use crate::preparation::PreparationMock;
use crate::state::{MachineState, Mode};
use crate::terminal_completion::{CompletionAction, UninstallCleanup};

#[test]
fn install_reads_legacy_endpoint_without_stopping_or_persisting_migration() {
    let files = IdentityFiles::new(legacy_state());
    let prepared = files
        .setup()
        .prepare(Action::Install {
            token: Secret::new("synthetic-install-token"),
            agent_url: None,
        })
        .unwrap();
    let ConfiguredAction::Install { endpoint, .. } = prepared.action else {
        panic!("install must keep its command endpoint");
    };
    assert_eq!(endpoint.as_str(), "wss://app.ciproxy.dev/agent/");
    assert_eq!(
        prepared.state.identity.unwrap().endpoint.unwrap().as_str(),
        "wss://preview.ciproxy.example/agent/"
    );
    assert_eq!(*files.effects.lock().unwrap(), Vec::<&str>::new());
    assert_eq!(*files.state.lock().unwrap(), legacy_state());
    assert_eq!(
        files.legacy.lock().unwrap().as_deref(),
        Some("wss://preview.ciproxy.example/agent/")
    );
}

#[test]
fn run_restores_endpoint_after_an_old_writer_drops_the_identity_field() {
    let files = IdentityFiles::new(legacy_state());
    files.setup().prepare(Action::Run).unwrap();
    *files.state.lock().unwrap() = legacy_state();
    let prepared = files.setup().prepare(Action::Run).unwrap();
    let ConfiguredAction::Run { endpoint } = prepared.action else {
        panic!("run must keep its daemon action");
    };
    assert_eq!(
        endpoint.as_ref().map(|endpoint| endpoint.as_str()),
        Some("wss://preview.ciproxy.example/agent/")
    );
    assert_eq!(
        files.legacy.lock().unwrap().as_deref(),
        Some("wss://preview.ciproxy.example/agent/")
    );
    assert_eq!(*files.state.lock().unwrap(), prepared.state);
    assert_eq!(*files.effects.lock().unwrap(), vec!["save", "save"]);
}

#[test]
fn owned_load_removes_stale_legacy_without_replacing_the_identity_route() {
    let mut registered = legacy_state();
    registered.identity.as_mut().unwrap().endpoint =
        Some("wss://retained.ciproxy.example/agent/".parse().unwrap());
    for (state, expected) in [
        (MachineState::default(), None),
        (registered, Some("wss://retained.ciproxy.example/agent/")),
    ] {
        let files = IdentityFiles::new(state);
        let prepared = files.setup().prepare(Action::Run).unwrap();
        let ConfiguredAction::Run { endpoint } = prepared.action else {
            panic!("run must keep the daemon action");
        };
        assert_eq!(
            endpoint.as_ref().map(|endpoint| endpoint.as_str()),
            expected
        );
        assert_eq!(*files.legacy.lock().unwrap(), None);
        assert_eq!(*files.effects.lock().unwrap(), vec!["remove"]);
    }
}

#[tokio::test]
async fn uninstall_retains_legacy_until_the_identity_is_cleared() {
    let files = IdentityFiles::new(legacy_state());
    let prepared = files.setup().prepare(Action::Uninstall).unwrap();
    assert_eq!(
        files.legacy.lock().unwrap().as_deref(),
        Some("wss://preview.ciproxy.example/agent/")
    );
    assert_eq!(*files.effects.lock().unwrap(), vec!["stop", "save"]);
    let mock = Arc::new(Unimock::new((
        VmsMock::networks
            .some_call(matching!())
            .returns(Ok(Vec::new())),
        PreparationMock::remove
            .some_call(matching!())
            .returns(Ok(())),
        ServiceMock::uninstall
            .some_call(matching!())
            .returns(Ok(())),
    )));
    let mut agent = controller(prepared.state, mock.as_ref().clone());
    agent.state.lock().await.mode = Mode::Removed;
    agent.store = files.store();
    assert!(
        agent
            .complete_removal(CompletionAction::Uninstall(UninstallCleanup {
                preparation: mock.clone(),
                service: mock.clone(),
            }))
            .await
            .unwrap()
    );
    assert_eq!(agent.state.lock().await.identity, None);
    assert_eq!(*files.legacy.lock().unwrap(), None);
    assert_eq!(
        *files.effects.lock().unwrap(),
        vec!["stop", "save", "clear", "remove"]
    );
}
