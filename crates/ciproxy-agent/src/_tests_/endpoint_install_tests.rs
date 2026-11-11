//! Preview registration and service restart share the durable install endpoint.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use ciproxy_protocol::identifiers::{MachineId, WorkspaceId};
use ciproxy_protocol::secret::Secret;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{Connection, HostMock, ServiceMock, StoreMock};
use crate::connection::WebSocketConnection;
use crate::connection_fixtures::websocket;
use crate::connection_socket::WebSocketDialerMock;
use crate::controller::AgentMock;
use crate::endpoint::{EndpointFilesMock, EndpointSettings, SavedEndpoint};
use crate::fixtures::report;
use crate::host_preparation::PreparationMock;
use crate::installation::{Install, Installer};
use crate::state::MachineState;

#[tokio::test]
async fn preview_install_endpoint_survives_service_restart_and_reinstall() {
    let preview = "wss://preview.ciproxy.example:8443/agent/";
    let supplied = preview.parse().unwrap();
    let saved = Arc::new(Mutex::new(None::<String>));
    let read = saved.clone();
    let write = saved.clone();
    let files = Arc::new(Unimock::new((
        EndpointFilesMock::read
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| Ok(read.lock().unwrap().clone()))),
        EndpointFilesMock::write
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, endpoint| {
                *write.lock().unwrap() = Some(endpoint.to_owned());
                Ok(())
            })),
    )));
    let settings = Arc::new(SavedEndpoint {
        files: files.clone(),
    });
    let endpoint = settings.resolve(Some(&supplied)).unwrap();
    let (registration, mut peer) = websocket().await;
    let (restarted, _peer) = websocket().await;
    let sockets = Mutex::new(VecDeque::from([registration, restarted]));
    let dialer = Arc::new(Unimock::new(
        WebSocketDialerMock::dial
            .some_call(matching!((url) if *url == "wss://preview.ciproxy.example:8443/agent/"))
            .answers_arc(Arc::new(move |_, _| {
                Ok(sockets.lock().unwrap().pop_front().unwrap())
            }))
            .n_times(2),
    ));
    let state = Arc::new(Mutex::new(MachineState::default()));
    let load = state.clone();
    let store = state.clone();
    let service_endpoints = Arc::new(Mutex::new(Vec::new()));
    let starts = service_endpoints.clone();
    let service_settings = saved.clone();
    let mock = Arc::new(Unimock::new((
        HostMock::require_root
            .each_call(matching!())
            .answers(&|_| Ok(())),
        PreparationMock::prepare
            .each_call(matching!())
            .answers(&|_| Ok(())),
        StoreMock::load
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| Ok(load.lock().unwrap().clone()))),
        StoreMock::save
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, value| {
                *store.lock().unwrap() = value.clone();
                Ok(())
            })),
        AgentMock::report
            .some_call(matching!())
            .returns(Ok(report().await)),
        ServiceMock::install
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| {
                starts
                    .lock()
                    .unwrap()
                    .push(service_settings.lock().unwrap().clone());
                Ok(())
            })),
    )));
    let connection = Arc::new(WebSocketConnection::new(
        endpoint.as_str().to_owned(),
        dialer.clone(),
    ));
    let mut installer = Installer {
        endpoint,
        endpoints: settings,
        store: mock.clone(),
        host: mock.clone(),
        service: mock.clone(),
        connection,
        agent: mock.clone(),
        preparation: mock,
    };
    let register = async {
        let message = peer.next().await.unwrap().unwrap();
        let message: AgentMessage = serde_json::from_str(message.to_text().unwrap()).unwrap();
        assert!(
            matches!(message, AgentMessage::Register { install_token, .. }
            if install_token.expose_secret() == "synthetic-install-token")
        );
        let response = ControlMessage::Registered {
            workspace_id: WorkspaceId("preview-workspace".to_owned()),
            machine_id: MachineId("preview-machine".to_owned()),
            credential: Secret::new("synthetic-machine-credential"),
        };
        peer.send(Message::Text(
            serde_json::to_string(&response).unwrap().into(),
        ))
        .await
        .unwrap();
    };
    let (result, ()) = tokio::join!(
        installer.install(Secret::new("synthetic-install-token")),
        register,
    );
    result.unwrap();
    assert_eq!(saved.lock().unwrap().as_deref(), Some(preview));
    assert_eq!(
        *service_endpoints.lock().unwrap(),
        vec![Some(preview.to_owned())]
    );

    let restarted = Arc::new(SavedEndpoint { files });
    let endpoint = restarted.resolve(None).unwrap();
    assert_eq!(endpoint.as_str(), preview);
    let connection = Arc::new(WebSocketConnection::new(
        endpoint.as_str().to_owned(),
        dialer,
    ));
    connection.connect().await.unwrap();

    installer.endpoint = endpoint;
    installer.endpoints = restarted;
    installer.connection = connection;
    installer
        .install(Secret::new("expired-install-token"))
        .await
        .unwrap();
    assert_eq!(saved.lock().unwrap().as_deref(), Some(preview));
    assert_eq!(
        *service_endpoints.lock().unwrap(),
        vec![Some(preview.to_owned()), Some(preview.to_owned())]
    );
}
