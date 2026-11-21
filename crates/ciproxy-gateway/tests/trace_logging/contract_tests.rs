//! TRACE keeps safe diagnostics and excludes serialized authentication secrets.

use std::sync::Arc;
use std::time::Duration;

use ciproxy_gateway::connections::{Connections, LocalConnections};
use ciproxy_gateway::logging;
use ciproxy_gateway::transport::{AgentTransport, WebSocketTransport};
use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use ciproxy_protocol::version::ProtocolVersion;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::runtime::Builder;
use tokio::time::timeout;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use tracing_subscriber::util::SubscriberInitExt;
use uuid::Uuid;

use super::capture::Capture;
use super::database::Database;
use super::messages::{hello, register, report};
use super::support::{INSTALL, MACHINE, WORKSPACE, gateway};

#[test]
fn hosted_trace_logging_hides_registration_and_hello_credentials() {
    const NATIVE: &str = "dummy-native-frame-secret";
    let capture = Capture::default();
    let directives = "trace,tungstenite=trace,tungstenite::protocol=trace,tungstenite::protocol::frame=trace,tokio_tungstenite=trace,tokio_tungstenite::compat=trace";
    let configured =
        logging::subscriber(Ok(directives.into()), BoxMakeWriter::new(capture.clone()));
    assert!(configured.notice.is_none());
    configured.subscriber.init();
    let database = Database::new();
    database.workspace(WORKSPACE);
    let connections: Arc<dyn Connections> = Arc::new(LocalConnections::default());
    let service = gateway(&database, connections.clone());
    let token = service.issue_token(WORKSPACE, WORKSPACE).unwrap();
    {
        let runtime = Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            let transport: &dyn AgentTransport = &WebSocketTransport::new(service, connections);
            let router = transport.router();
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let url = format!("ws://{}/agent/", listener.local_addr().unwrap());
            let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap(); });
            let mut first = connect_async(&url).await.unwrap().0;
            let registration = register(token.token, report());
            first.send(Message::Text(serde_json::to_string(&registration).unwrap().into())).await.unwrap();
            let frame = timeout(Duration::from_secs(10), first.next()).await.unwrap().unwrap().unwrap();
            let registered: ControlMessage = serde_json::from_str(frame.to_text().unwrap()).unwrap();
            let ControlMessage::Registered { workspace_id, machine_id, credential } = registered else { panic!("registration failed"); };
            let workspace = Uuid::parse_str(&workspace_id.0).unwrap();
            assert_eq!(workspace, WORKSPACE);
            first.close(None).await.unwrap();
            let mut second = connect_async(&url).await.unwrap().0;
            let authentication: AgentMessage = hello(workspace, machine_id, credential, 1, report());
            second.send(Message::Text(serde_json::to_string(&authentication).unwrap().into())).await.unwrap();
            let frame = timeout(Duration::from_secs(10), second.next()).await.unwrap().unwrap().unwrap();
            let welcome: ControlMessage = serde_json::from_str(frame.to_text().unwrap()).unwrap();
            assert_eq!(welcome, ControlMessage::Welcome { protocol_version: ProtocolVersion::CURRENT });
            second.close(None).await.unwrap();
            tracing::trace!(target: "ciproxy_trace_probe", "safe native trace");
            log::trace!(target: "ciproxy_trace_probe", "safe bridged trace");
            tracing::trace!(target: "tungstenite::protocol::frame", secret = NATIVE, "unsafe native frame");
            tokio::spawn(async { tracing::trace!(target: "ciproxy_trace_probe", "safe spawned trace"); }).await.unwrap();
            tokio::task::spawn_blocking(|| { tracing::trace!(target: "ciproxy_trace_probe", "safe blocking trace"); }).await.unwrap();
            server.abort();
            assert!(server.await.unwrap_err().is_cancelled());
        });
    }
    let output = capture.text();
    assert!(output.contains("safe native trace"));
    assert!(output.contains("safe bridged trace"));
    assert!(output.contains("safe spawned trace"));
    assert!(output.contains("safe blocking trace"));
    assert_eq!(
        (
            output.contains(INSTALL),
            output.contains(MACHINE),
            output.contains(NATIVE)
        ),
        (false, false, false),
        "authentication or native frame secrets reached hosted logs"
    );
}
