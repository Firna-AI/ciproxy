//! Real pinned WebSocket traffic cannot expose credentials through production logs.

use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::identifiers::{MachineId, WorkspaceId};
use ciproxy_protocol::secret::Secret;
use ciproxy_protocol::version::ProtocolVersion;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::io::duplex;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::{Message, protocol::Role};
use tracing::instrument::WithSubscriber;
use tracing_subscriber::fmt::writer::BoxMakeWriter;

use crate::fixtures::report;
use crate::logging::subscriber;

const MACHINE_CREDENTIAL: &str = "TRACE_DUMMY_MACHINE_CREDENTIAL_11";
const INSTALL_TOKEN: &str = "TRACE_DUMMY_INSTALL_TOKEN_11";

#[derive(Clone)]
struct Capture(Arc<Mutex<Vec<u8>>>);

impl Write for Capture {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Deserialize)]
struct Record {
    level: String,
    target: String,
    fields: Fields,
}

#[derive(Deserialize)]
struct Fields {
    message: Option<String>,
}

#[tokio::test]
async fn trace_websocket_traffic_never_logs_install_or_machine_credentials() {
    tracing_log::LogTracer::init().unwrap();
    let snapshot = report().await;
    for directives in [
        "trace",
        "trace,tungstenite=trace,tungstenite::protocol=trace,tungstenite::protocol::frame=trace,tokio_tungstenite=trace,tokio_tungstenite::compat=trace",
        "off,log=trace,ciproxy_agent::logging_probe=trace,ciproxy_agent::logging_bridge=trace,tungstenite::protocol=trace,tungstenite::protocol::frame=trace,tokio_tungstenite::compat=trace",
    ] {
        let capture = Capture(Arc::new(Mutex::new(Vec::new())));
        let writer = capture.clone();
        let logging = subscriber(BoxMakeWriter::new(move || writer.clone()), directives);
        let register = AgentMessage::Register {
            protocol_version: ProtocolVersion::CURRENT,
            workspace_id: WorkspaceId("logging-workspace".to_owned()),
            install_token: Secret::new(INSTALL_TOKEN),
            report: snapshot.clone(),
        };
        let authenticate = AgentMessage::Hello {
            protocol_version: ProtocolVersion::CURRENT,
            workspace_id: WorkspaceId("logging-workspace".to_owned()),
            machine_id: MachineId("logging-machine".to_owned()),
            credential: Secret::new(MACHINE_CREDENTIAL),
            report: snapshot.clone(),
        };
        async move {
            tracing::trace!(target: "ciproxy_agent::logging_probe", "safe_native_trace_marker");
            log::trace!(target: "ciproxy_agent::logging_bridge", "safe_bridge_trace_marker");
            tracing::trace!(target: "tungstenite::protocol", "native_transport_namespace_probe");
            tracing::trace!(target: "tokio_tungstenite::compat", "native_async_transport_namespace_probe");
            let (client, server) = duplex(64 * 1024);
            let mut client = WebSocketStream::from_raw_socket(client, Role::Client, None).await;
            let mut server = WebSocketStream::from_raw_socket(server, Role::Server, None).await;
            for message in [register, authenticate] {
                let json = serde_json::to_string(&message).unwrap();
                client.send(Message::Text(json.into())).await.unwrap();
                let received = server.next().await.unwrap().unwrap().into_text().unwrap();
                assert_eq!(serde_json::from_str::<AgentMessage>(&received).unwrap(), message);
            }
        }.with_subscriber(logging).await;
        let output = String::from_utf8(capture.0.lock().unwrap().clone()).unwrap();
        let records = output
            .lines()
            .map(|line| (line, serde_json::from_str::<Record>(line).unwrap()))
            .collect::<Vec<_>>();
        assert_eq!(
            records
                .iter()
                .filter(|(_, record)| record.level == "TRACE"
                    && record.fields.message.as_deref() == Some("safe_native_trace_marker"))
                .count(),
            1
        );
        assert_eq!(
            records
                .iter()
                .filter(|(_, record)| record.level == "TRACE"
                    && record.target == "ciproxy_agent::logging_bridge"
                    && record.fields.message.as_deref() == Some("safe_bridge_trace_marker"))
                .count(),
            1,
            "the real log-to-tracing bridge must be active"
        );
        let machine_records = records
            .iter()
            .filter(|(line, _)| line.contains(MACHINE_CREDENTIAL))
            .count();
        let install_records = records
            .iter()
            .filter(|(line, _)| line.contains(INSTALL_TOKEN))
            .count();
        assert_eq!(
            (machine_records, install_records),
            (0, 0),
            "production logging exposed dummy machine/install credentials"
        );
        assert_eq!(
            records
                .iter()
                .filter(|(_, record)| record.target.starts_with("tungstenite")
                    || record.target.starts_with("tokio_tungstenite"))
                .count(),
            0
        );
    }
}
