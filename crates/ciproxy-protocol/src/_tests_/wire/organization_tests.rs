//! Organization scope before assignment and repository facts after assignment.

use serde_json::json;

use crate::agent::{AgentMessage, ControlMessage, JobPreparationOutcome};
use crate::guest::AgentToGuest;
use crate::secret::Secret;
use crate::version::ProtocolVersion;

use super::fixtures;

#[test]
fn runner_scope_retains_organization_and_defers_repository_until_assignment() {
    let start = serde_json::to_value(ControlMessage::StartRunner {
        command_id: fixtures::command_id(),
        runner: fixtures::runner(),
    })
    .unwrap();
    let stop = serde_json::to_value(ControlMessage::StopRunner {
        command_id: fixtures::command_id(),
        organization_id: fixtures::organization_id(),
        runner_id: fixtures::runner_id(),
    })
    .unwrap();
    let guest = serde_json::to_value(AgentToGuest::ConfigureRunner {
        protocol_version: ProtocolVersion::CURRENT,
        runner_id: fixtures::runner_id(),
        organization_id: fixtures::organization_id(),
        registration: Secret::new(fixtures::REGISTRATION),
        cache: Some(fixtures::cache_connection()),
    })
    .unwrap();
    let hello = serde_json::to_value(AgentMessage::Hello {
        protocol_version: ProtocolVersion::CURRENT,
        workspace_id: fixtures::workspace_id(),
        machine_id: fixtures::machine_id(),
        credential: Secret::new(fixtures::CREDENTIAL),
        report: fixtures::report(),
    })
    .unwrap();
    let scopes = [
        ("runner start", &start["runner"], "organization-1"),
        ("idle runner stop", &stop, "organization-1"),
        ("guest configuration", &guest, "organization-1"),
        (
            "warm runner after reconnect",
            &hello["report"]["runners"][0],
            "organization-2",
        ),
        (
            "running job after reconnect",
            &hello["report"]["runners"][1],
            "organization-1",
        ),
    ];
    for (name, scope, expected) in scopes {
        assert_eq!(scope["organization_id"], json!(expected), "{name}");
    }
    let unassigned = [
        ("runner start", &start["runner"]),
        ("guest configuration", &guest),
        (
            "warm runner after reconnect",
            &hello["report"]["runners"][0],
        ),
    ];
    for (name, scope) in unassigned {
        assert_eq!(scope.get("repository"), None, "{name}");
    }
}

#[test]
fn assigned_job_facts_preserve_organization_and_actual_repository() {
    let control = serde_json::to_value(ControlMessage::JobPreparation {
        command_id: fixtures::command_id(),
        runner_id: fixtures::runner_id(),
        job_id: fixtures::job_id(),
        outcome: JobPreparationOutcome::TrustedFacts {
            facts: Box::new(fixtures::facts()),
        },
    })
    .unwrap();
    let guest = serde_json::to_value(AgentToGuest::JobFacts {
        job_id: fixtures::job_id(),
        facts: fixtures::facts(),
    })
    .unwrap();
    for (name, message) in [
        ("control-plane facts", control["outcome"].clone()),
        ("guest facts", guest),
    ] {
        assert_eq!(
            message["facts"]["organization_id"],
            json!("organization-1"),
            "{name}"
        );
        assert_eq!(
            message["facts"]["repository"],
            json!({
                "id": "repository-1", "full_name": "example/project",
            }),
            "{name}"
        );
    }
}
