//! Absence evidence must follow the agent's first confirmation.

use std::sync::{Arc, Mutex};

use chrono::Duration;
use ciproxy_control_plane::runtime::OwnMachines;
use ciproxy_protocol::agent::{AgentMessage, CommandFailure, CommandOutcome, ControlMessage};
use ciproxy_protocol::identifiers::CommandId;
use ciproxy_store_interface::runners::RunnerState;
use unimock::Unimock;

use super::recovery_support::{gateway, keep_warm, runtime, saved};
use super::support::{Fixture, at};

#[tokio::test]
async fn event_contact_cannot_make_an_old_absent_report_release_a_busy_runner() {
    let mut fixture = Fixture::new();
    fixture.contact.report.runners.clear();
    fixture.contact.at = at() - Duration::minutes(2);
    fixture
        .store
        .record_contact(fixture.workspace, fixture.machine, &fixture.contact)
        .unwrap();
    fixture
        .store
        .update_runner(
            fixture.workspace,
            fixture.organization,
            fixture.intent.runner.id,
            RunnerState::Busy,
            Some(47),
        )
        .unwrap();
    fixture
        .store
        .record_activity(
            fixture.workspace,
            fixture.machine,
            fixture.contact.session_id,
            at(),
        )
        .unwrap();
    let messages = Arc::new(Mutex::new(Vec::new()));
    let runtime = runtime(&fixture, Arc::new(Unimock::new(())), gateway(messages));
    runtime
        .agent(
            fixture.workspace,
            fixture.machine,
            AgentMessage::CommandResult {
                command_id: CommandId(fixture.intent.command_id.to_string()),
                outcome: CommandOutcome::Applied,
            },
        )
        .await
        .unwrap();
    runtime.tick().await.unwrap();
    assert_eq!(saved(&fixture).runner.state, RunnerState::Busy);
}

#[tokio::test]
async fn inventory_after_issue_but_before_agent_confirmation_cannot_retire_the_start() {
    let mut fixture = Fixture::new();
    fixture
        .store
        .retire_own_runner(
            fixture.workspace,
            fixture.intent.runner.id,
            fixture.intent.command_id,
            at() - Duration::seconds(2),
        )
        .unwrap();
    fixture
        .store
        .update_runner(
            fixture.workspace,
            fixture.organization,
            fixture.intent.runner.id,
            RunnerState::Starting,
            Some(47),
        )
        .unwrap();
    fixture.contact.report.runners.clear();
    fixture.contact.at = at() - Duration::seconds(1);
    fixture
        .store
        .record_contact(fixture.workspace, fixture.machine, &fixture.contact)
        .unwrap();
    keep_warm(&fixture);
    let messages = Arc::new(Mutex::new(Vec::new()));
    let runtime = runtime(
        &fixture,
        Arc::new(Unimock::new(())),
        gateway(messages.clone()),
    );
    runtime
        .agent(
            fixture.workspace,
            fixture.machine,
            AgentMessage::CommandResult {
                command_id: CommandId(fixture.intent.command_id.to_string()),
                outcome: CommandOutcome::Applied,
            },
        )
        .await
        .unwrap();
    runtime.tick().await.unwrap();
    assert_eq!(saved(&fixture).runner.state, RunnerState::Starting);
    assert_eq!(
        messages
            .lock()
            .unwrap()
            .iter()
            .filter(|message| matches!(message, ControlMessage::StopRunner { .. }))
            .count(),
        0
    );
}

#[tokio::test]
async fn unconfirmed_stop_waits_for_a_result_then_releases_on_later_inventory() {
    let mut fixture = Fixture::new();
    fixture
        .store
        .retire_own_runner(
            fixture.workspace,
            fixture.intent.runner.id,
            fixture.intent.command_id,
            at() - Duration::seconds(2),
        )
        .unwrap();
    fixture.contact.report.runners.clear();
    fixture.contact.at = at() - Duration::seconds(1);
    fixture
        .store
        .record_contact(fixture.workspace, fixture.machine, &fixture.contact)
        .unwrap();
    let messages = Arc::new(Mutex::new(Vec::new()));
    let runtime = runtime(
        &fixture,
        Arc::new(Unimock::new(())),
        gateway(messages.clone()),
    );
    runtime.tick().await.unwrap();
    assert_eq!(saved(&fixture).runner.state, RunnerState::Stopping);
    assert_eq!(saved(&fixture).confirmed_at, None);
    runtime
        .agent(
            fixture.workspace,
            fixture.machine,
            AgentMessage::CommandResult {
                command_id: CommandId(fixture.intent.command_id.to_string()),
                outcome: CommandOutcome::Failed {
                    reason: CommandFailure::RunnerStop,
                },
            },
        )
        .await
        .unwrap();
    assert_eq!(saved(&fixture).confirmed_at, Some(at()));
    assert!(!saved(&fixture).acknowledged);
    fixture.contact.at = at() + Duration::seconds(1);
    fixture
        .store
        .record_contact(fixture.workspace, fixture.machine, &fixture.contact)
        .unwrap();
    runtime.tick().await.unwrap();
    assert_eq!(saved(&fixture).runner.state, RunnerState::Stopped);
    assert_eq!(
        messages
            .lock()
            .unwrap()
            .iter()
            .filter(|message| matches!(message, ControlMessage::StopRunner { .. }))
            .count(),
        1
    );
}
