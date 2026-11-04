//! Workspace identity across machine registration and authentication.

use serde_json::json;

use crate::agent::{AgentMessage, ControlMessage};
use crate::secret::Secret;
use crate::version::ProtocolVersion;

use super::fixtures;

#[test]
fn workspace_identity_crosses_registration_reply_and_authenticated_hello() {
    let register = AgentMessage::Register {
        protocol_version: ProtocolVersion::CURRENT,
        workspace_id: fixtures::workspace_id(),
        install_token: Secret::new(fixtures::INSTALL_TOKEN),
        report: fixtures::report(),
    };
    let registered = ControlMessage::Registered {
        workspace_id: fixtures::workspace_id(),
        machine_id: fixtures::machine_id(),
        credential: Secret::new(fixtures::CREDENTIAL),
    };
    let hello = AgentMessage::Hello {
        protocol_version: ProtocolVersion::CURRENT,
        workspace_id: fixtures::workspace_id(),
        machine_id: fixtures::machine_id(),
        credential: Secret::new(fixtures::CREDENTIAL),
        report: fixtures::report(),
    };
    let messages = [
        ("register", serde_json::to_value(register).unwrap()),
        ("registered", serde_json::to_value(registered).unwrap()),
        ("hello", serde_json::to_value(hello).unwrap()),
    ];
    for (name, message) in messages {
        assert_eq!(message["workspace_id"], json!("workspace-1"), "{name}");
    }
}
