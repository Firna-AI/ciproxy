//! Registration saves the gateway identity. Reinstall preserves identity and drain.

use std::sync::{Arc, Mutex};

use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use ciproxy_protocol::identifiers::{MachineId, WorkspaceId};
use ciproxy_protocol::secret::Secret;
use ciproxy_protocol::version::ProtocolVersion;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{ConnectionMock, HostMock, ServiceMock, StoreMock};
use crate::connection_types::ControlEvent;
use crate::controller::AgentMock;
use crate::endpoint::{AgentEndpoint, EndpointSettingsMock};
use crate::error::Error;
use crate::fixtures::{identity, report, vm};
use crate::host_preparation::PreparationMock;
use crate::installation::{Install, Installer};
use crate::state::{Identity, MachineState, Mode};

fn installer(mock: Unimock) -> Installer {
    let mock = Arc::new(mock);
    Installer {
        endpoint: AgentEndpoint::default(),
        endpoints: Arc::new(Unimock::new(
            EndpointSettingsMock::save
                .each_call(matching!(_))
                .answers(&|_, _| Ok(())),
        )),
        store: mock.clone(),
        host: mock.clone(),
        service: mock.clone(),
        connection: mock.clone(),
        agent: mock.clone(),
        preparation: mock.clone(),
    }
}

#[tokio::test]
async fn reinstall_keeps_registration_without_reusing_the_install_token() {
    let state = MachineState {
        registration_claims: Default::default(),
        failed_starts: Default::default(),
        pending_networks: Default::default(),
        docker_budget: None,
        identity: Some(identity()),
        mode: Mode::Draining,
        runners: vec![vm("active-job", true)],
    };
    let events = Arc::new(Mutex::new(Vec::new()));
    let prepared = events.clone();
    let installed = events.clone();
    let mock = Unimock::new((
        PreparationMock::prepare
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                prepared.lock().unwrap().push("prepare");
                Ok(())
            })),
        HostMock::require_root
            .some_call(matching!())
            .returns(Ok(())),
        StoreMock::load.some_call(matching!()).returns(Ok(state)),
        ServiceMock::install
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                installed.lock().unwrap().push("service");
                Ok(())
            })),
    ));
    let installer = installer(mock);
    installer
        .install(Secret::new("expired-install-token"))
        .await
        .unwrap();
    assert_eq!(*events.lock().unwrap(), vec!["prepare", "service"]);
}

#[tokio::test]
async fn registration_saves_the_workspace_and_credential_returned_by_gateway() {
    let report = report().await;
    let initial_report = report.clone();
    let saved = Arc::new(Mutex::new(Vec::new()));
    let observations = saved.clone();
    let sent = Arc::new(Mutex::new(Vec::new()));
    let messages = sent.clone();
    let mock = Unimock::new((
        PreparationMock::prepare
            .some_call(matching!())
            .returns(Ok(())),
        HostMock::require_root
            .some_call(matching!())
            .returns(Ok(())),
        StoreMock::load
            .some_call(matching!())
            .returns(Ok(MachineState::default())),
        StoreMock::save
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, state| {
                observations.lock().unwrap().push(state.clone());
                Ok(())
            })),
        AgentMock::report.some_call(matching!()).returns(Ok(report)),
        ConnectionMock::connect
            .some_call(matching!())
            .returns(Ok(())),
        ConnectionMock::send
            .some_call(matching!(_))
            .answers_arc(Arc::new(move |_, message| {
                messages.lock().unwrap().push(message.clone());
                Ok(())
            })),
        ConnectionMock::receive_handshake
            .some_call(matching!())
            .returns(Ok(ControlEvent::Message(Box::new(
                ControlMessage::Registered {
                    workspace_id: WorkspaceId("gateway-workspace-7b3".to_owned()),
                    machine_id: MachineId("gateway-machine-c8".to_owned()),
                    credential: Secret::new("gateway-machine-secret"),
                },
            )))),
        ServiceMock::install.some_call(matching!()).returns(Ok(())),
    ));
    installer(mock)
        .install(Secret::new("install-secret"))
        .await
        .unwrap();
    assert_eq!(
        *sent.lock().unwrap(),
        vec![AgentMessage::Register {
            protocol_version: ProtocolVersion::CURRENT,
            install_token: Secret::new("install-secret"),
            report: initial_report,
        }]
    );
    let states = saved.lock().unwrap();
    assert_eq!(
        states.last().unwrap().identity,
        Some(Identity {
            workspace_id: WorkspaceId("gateway-workspace-7b3".to_owned()),
            machine_id: MachineId("gateway-machine-c8".to_owned()),
            credential: Secret::new("gateway-machine-secret"),
        })
    );
    assert!(states.iter().all(|state| {
        !serde_json::to_string(state)
            .unwrap()
            .contains("install-secret")
    }));
}

#[tokio::test]
async fn registration_rejects_a_wrong_handshake_without_saving_identity() {
    let report = report().await;
    let saved = Arc::new(Mutex::new(Vec::new()));
    let observations = saved.clone();
    let mock = Unimock::new((
        PreparationMock::prepare
            .some_call(matching!())
            .returns(Ok(())),
        HostMock::require_root
            .some_call(matching!())
            .returns(Ok(())),
        StoreMock::load
            .some_call(matching!())
            .returns(Ok(MachineState::default())),
        StoreMock::save
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, state| {
                observations.lock().unwrap().push(state.clone());
                Ok(())
            })),
        AgentMock::report.some_call(matching!()).returns(Ok(report)),
        ConnectionMock::connect
            .some_call(matching!())
            .returns(Ok(())),
        ConnectionMock::send.some_call(matching!(_)).returns(Ok(())),
        ConnectionMock::receive_handshake
            .some_call(matching!())
            .returns(Ok(ControlEvent::Message(Box::new(
                ControlMessage::Welcome {
                    protocol_version: ProtocolVersion::CURRENT,
                },
            )))),
    ));
    assert!(matches!(
        installer(mock).install(Secret::new("install-secret")).await,
        Err(Error::IdentityMismatch)
    ));
    assert_eq!(*saved.lock().unwrap(), vec![MachineState::default()]);
}
