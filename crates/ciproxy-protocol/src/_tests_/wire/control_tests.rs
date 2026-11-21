//! Gateway replies and commands sent to an own machine.

use crate::agent::{ControlMessage, JobPreparationOutcome};
use crate::secret::Secret;
use crate::version::ProtocolVersion;

use super::assertions::snapshot;
use super::fixtures;

#[test]
fn registered_delivers_a_machine_specific_identity_and_credential() {
    snapshot(
        "control_registered",
        "registered",
        ControlMessage::Registered {
            workspace_id: fixtures::workspace_id(),
            machine_id: fixtures::machine_id(),
            credential: Secret::new(fixtures::CREDENTIAL),
        },
    );
}

#[test]
fn welcome_carries_the_agreed_protocol_version() {
    snapshot(
        "control_welcome",
        "welcome",
        ControlMessage::Welcome {
            protocol_version: ProtocolVersion::CURRENT,
        },
    );
}

#[test]
fn start_runner_delivers_size_label_and_one_use_registration() {
    snapshot(
        "control_start_runner",
        "start_runner",
        ControlMessage::StartRunner {
            command_id: fixtures::command_id(),
            runner: fixtures::runner(),
        },
    );
}

#[test]
fn stop_runner_targets_one_idle_runner() {
    snapshot(
        "control_stop_runner",
        "stop_runner",
        ControlMessage::StopRunner {
            command_id: fixtures::command_id(),
            organization_id: fixtures::organization_id(),
            runner_id: fixtures::runner_id(),
        },
    );
}

#[test]
fn job_facts_include_fork_default_and_base_ref_inputs() {
    snapshot(
        "control_job_preparation_facts",
        "job_preparation",
        ControlMessage::JobPreparation {
            command_id: fixtures::command_id(),
            runner_id: fixtures::runner_id(),
            job_id: fixtures::job_id(),
            outcome: JobPreparationOutcome::TrustedFacts {
                facts: Box::new(fixtures::facts()),
            },
        },
    );
}

#[test]
fn github_only_preparation_carries_organization_without_inventing_facts() {
    snapshot(
        "control_job_preparation_github_only",
        "job_preparation",
        ControlMessage::JobPreparation {
            command_id: fixtures::command_id(),
            runner_id: fixtures::runner_id(),
            job_id: fixtures::job_id(),
            outcome: JobPreparationOutcome::GitHubOnly {
                organization_id: fixtures::organization_id(),
            },
        },
    );
}

#[test]
fn drain_has_a_correlated_command_identity() {
    snapshot(
        "control_drain",
        "drain",
        ControlMessage::Drain {
            command_id: fixtures::command_id(),
        },
    );
}

#[test]
fn resume_has_a_correlated_command_identity() {
    snapshot(
        "control_resume",
        "resume",
        ControlMessage::Resume {
            command_id: fixtures::command_id(),
        },
    );
}

#[test]
fn remove_has_a_correlated_command_identity() {
    snapshot(
        "control_remove",
        "remove",
        ControlMessage::Remove {
            command_id: fixtures::command_id(),
        },
    );
}

#[test]
fn clear_cache_does_not_name_a_docker_disk_or_github_cache() {
    snapshot(
        "control_clear_cache",
        "clear_cache",
        ControlMessage::ClearCache {
            command_id: fixtures::command_id(),
        },
    );
}

#[test]
fn fetch_image_carries_a_signed_url_and_checksum() {
    snapshot(
        "control_fetch_image",
        "fetch_image",
        ControlMessage::FetchImage {
            command_id: fixtures::command_id(),
            image: fixtures::image(),
        },
    );
}
