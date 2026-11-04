//! Reinstall preserves identity and new registration binds to the resolved workspace.

use std::sync::{Arc, Mutex};

use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use ciproxy_protocol::identifiers::{MachineId, WorkspaceId};
use ciproxy_protocol::secret::Secret;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{ConnectionMock, HostMock, ServiceMock, StoreMock, WorkspaceResolverMock};
use crate::controller::AgentMock;
use crate::error::Error;
use crate::fixtures::{identity, report};
use crate::installation::{Install, Installer};
use crate::state::{MachineState, Mode};

fn installer(mock: Unimock) -> Installer {
    let mock = Arc::new(mock);
    Installer {
        store: mock.clone(),
        host: mock.clone(),
        service: mock.clone(),
        resolver: mock.clone(),
        connection: mock.clone(),
        agent: mock,
    }
}

#[tokio::test]
async fn reinstall_keeps_registration_without_reusing_the_install_token() {
    let state = MachineState {
        identity: Some(identity()),
        mode: Mode::Draining,
        ..MachineState::default()
    };
    let mock = Unimock::new((
        HostMock::require_root
            .some_call(matching!())
            .returns(Ok(())),
        StoreMock::load.some_call(matching!()).returns(Ok(state)),
        ServiceMock::install.some_call(matching!()).returns(Ok(())),
    ));
    installer(mock)
        .install(Secret::new("expired-install-token"))
        .await
        .unwrap();
}

#[tokio::test]
async fn registration_saves_only_the_workspace_bound_machine_credential() {
    let report = report().await;
    let saved = Arc::new(Mutex::new(Vec::new()));
    let observations = saved.clone();
    let mock = Unimock::new((
        HostMock::require_root.some_call(matching!()).returns(Ok(())),
        StoreMock::load.some_call(matching!()).returns(Ok(MachineState::default())),
        WorkspaceResolverMock::workspace.some_call(matching!((token) if token.expose_secret() == "install-secret")).returns(Ok(WorkspaceId("workspace-1".to_owned()))),
        StoreMock::save.each_call(matching!(_)).answers_arc(Arc::new(move |_, state| {
            assert!(!serde_json::to_string(state).unwrap().contains("install-secret"));
            observations.lock().unwrap().push(state.clone());
            Ok(())
        })),
        AgentMock::report.some_call(matching!()).returns(Ok(report)),
        ConnectionMock::connect.some_call(matching!()).returns(Ok(())),
        ConnectionMock::send.some_call(matching!((message) if matches!(message, AgentMessage::Register { workspace_id, install_token, .. }
            if workspace_id.0 == "workspace-1" && install_token.expose_secret() == "install-secret"))).returns(Ok(())),
        ConnectionMock::receive_handshake.some_call(matching!()).returns(Ok(ControlMessage::Registered {
            workspace_id: WorkspaceId("workspace-1".to_owned()), machine_id: MachineId("machine-1".to_owned()), credential: Secret::new("machine-secret"),
        })),
        ServiceMock::install.some_call(matching!()).returns(Ok(())),
    ));
    installer(mock)
        .install(Secret::new("install-secret"))
        .await
        .unwrap();
    assert_eq!(
        saved.lock().unwrap().last().unwrap().identity,
        Some(identity())
    );
}

#[tokio::test]
async fn registration_does_not_save_a_credential_for_another_workspace() {
    let report = report().await;
    let mock = Unimock::new((
        HostMock::require_root
            .some_call(matching!())
            .returns(Ok(())),
        StoreMock::load
            .some_call(matching!())
            .returns(Ok(MachineState::default())),
        WorkspaceResolverMock::workspace
            .some_call(matching!(_))
            .returns(Ok(WorkspaceId("workspace-1".to_owned()))),
        StoreMock::save
            .some_call(matching!((state) if state.identity.is_none()))
            .returns(Ok(())),
        AgentMock::report.some_call(matching!()).returns(Ok(report)),
        ConnectionMock::connect
            .some_call(matching!())
            .returns(Ok(())),
        ConnectionMock::send.some_call(matching!(_)).returns(Ok(())),
        ConnectionMock::receive_handshake
            .some_call(matching!())
            .returns(Ok(ControlMessage::Registered {
                workspace_id: WorkspaceId("workspace-2".to_owned()),
                machine_id: MachineId("machine-1".to_owned()),
                credential: Secret::new("wrong-credential"),
            })),
    ));
    assert!(matches!(
        installer(mock).install(Secret::new("install-secret")).await,
        Err(Error::IdentityMismatch)
    ));
}
