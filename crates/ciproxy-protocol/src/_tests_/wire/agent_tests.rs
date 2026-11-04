//! Agent-originated JSON, including registration and reconnect state.

use crate::agent::{AgentMessage, CommandFailure, CommandOutcome};
use crate::runners::RunnerExit;
use crate::secret::Secret;
use crate::version::ProtocolVersion;

use super::assertions::snapshot;
use super::fixtures;

#[test]
fn register_has_install_token_and_initial_state() {
    snapshot(
        "agent_register",
        "register",
        AgentMessage::Register {
            protocol_version: ProtocolVersion::CURRENT,
            install_token: Secret::new(fixtures::INSTALL_TOKEN),
            report: fixtures::report(),
        },
    );
}

#[test]
fn hello_authenticates_and_restores_current_runner_state() {
    snapshot(
        "agent_hello",
        "hello",
        AgentMessage::Hello {
            protocol_version: ProtocolVersion::CURRENT,
            workspace_id: fixtures::workspace_id(),
            machine_id: fixtures::machine_id(),
            credential: Secret::new(fixtures::CREDENTIAL),
            report: fixtures::report(),
        },
    );
}

#[test]
fn report_carries_slots_health_and_eligible_cache_counts() {
    snapshot(
        "agent_report",
        "report",
        AgentMessage::Report {
            report: fixtures::report(),
        },
    );
}

#[test]
fn runner_ready_identifies_the_new_warm_runner() {
    snapshot(
        "agent_runner_ready",
        "runner_ready",
        AgentMessage::RunnerReady {
            runner_id: fixtures::runner_id(),
        },
    );
}

#[test]
fn job_started_identifies_the_actual_github_assignment() {
    snapshot(
        "agent_job_started",
        "job_started",
        AgentMessage::JobStarted {
            runner_id: fixtures::runner_id(),
            job_id: fixtures::job_id(),
        },
    );
}

#[test]
fn runner_exited_reports_a_process_without_an_exit_code() {
    snapshot(
        "agent_runner_exited",
        "runner_exited",
        AgentMessage::RunnerExited {
            runner_id: fixtures::runner_id(),
            exit: RunnerExit { code: None },
        },
    );
}

#[test]
fn command_result_reports_a_typed_failure_without_job_output() {
    snapshot(
        "agent_command_result",
        "command_result",
        AgentMessage::CommandResult {
            command_id: fixtures::command_id(),
            outcome: CommandOutcome::Failed {
                reason: CommandFailure::ImageVerification,
            },
        },
    );
}
