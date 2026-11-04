//! Redaction through every secret-bearing public message and nested value.

use std::fmt::Debug;

use serde::Serialize;

use crate::agent::{AgentMessage, ControlMessage};
use crate::guest::AgentToGuest;
use crate::secret::Secret;
use crate::version::ProtocolVersion;
use crate::wire_tests::fixtures;

fn assert_hidden<T: Debug + Serialize>(message: T, secrets: &[&str]) {
    let debug = format!("{message:?}");
    let json = serde_json::to_string(&message).unwrap();
    for secret in secrets {
        assert!(!debug.contains(secret), "Debug exposed {secret}: {debug}");
        assert!(
            json.contains(secret),
            "JSON did not deliver {secret}: {json}"
        );
    }
    assert!(debug.contains("[REDACTED]"), "missing redaction: {debug}");
}

#[test]
fn nested_debug_hides_secrets_while_json_delivers_them() {
    assert_hidden(
        AgentMessage::Register {
            protocol_version: ProtocolVersion::CURRENT,
            install_token: Secret::new(fixtures::INSTALL_TOKEN),
            report: fixtures::report(),
        },
        &[fixtures::INSTALL_TOKEN],
    );
    assert_hidden(
        AgentMessage::Hello {
            protocol_version: ProtocolVersion::CURRENT,
            workspace_id: fixtures::workspace_id(),
            machine_id: fixtures::machine_id(),
            credential: Secret::new(fixtures::CREDENTIAL),
            report: fixtures::report(),
        },
        &[fixtures::CREDENTIAL],
    );
    assert_hidden(
        ControlMessage::Registered {
            workspace_id: fixtures::workspace_id(),
            machine_id: fixtures::machine_id(),
            credential: Secret::new(fixtures::CREDENTIAL),
        },
        &[fixtures::CREDENTIAL],
    );
    assert_hidden(
        ControlMessage::StartRunner {
            command_id: fixtures::command_id(),
            runner: fixtures::runner(),
        },
        &[fixtures::REGISTRATION],
    );
    assert_hidden(
        ControlMessage::FetchImage {
            command_id: fixtures::command_id(),
            image: fixtures::image(),
        },
        &[fixtures::DOWNLOAD_URL],
    );
    assert_hidden(
        AgentToGuest::ConfigureRunner {
            protocol_version: ProtocolVersion::CURRENT,
            runner_id: fixtures::runner_id(),
            organization_id: fixtures::organization_id(),
            registration: Secret::new(fixtures::REGISTRATION),
            cache: Some(fixtures::cache_connection()),
        },
        &[
            fixtures::REGISTRATION,
            fixtures::CACHE_URL,
            fixtures::CACHE_TOKEN,
        ],
    );
}
