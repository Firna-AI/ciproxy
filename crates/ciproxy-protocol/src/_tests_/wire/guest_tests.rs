//! Both guest vsock directions and the Boat sample report.

use crate::cache::{DockerDiskAccess, DockerDiskAttachment};
use crate::guest::HANDOFF_PROTOCOL_VERSION;
use crate::guest::{AgentToGuest, GuestToAgent, JobDecision};
use crate::runners::RunnerExit;
use crate::secret::Secret;
use crate::usage::JobStepTiming;
use crate::version::ProtocolVersion;

use super::assertions::snapshot;
use super::fixtures;

#[test]
fn configure_runner_delivers_registration_over_vsock() {
    let configuration = AgentToGuest::ConfigureRunner {
        protocol_version: ProtocolVersion::CURRENT,
        runner_id: fixtures::runner_id(),
        organization_id: fixtures::organization_id(),
        registration: Secret::new(fixtures::REGISTRATION),
    };
    assert_eq!(
        serde_json::to_value(&configuration).unwrap().get("cache"),
        None
    );
    snapshot("vsock_configure_runner", "configure_runner", configuration);
}

#[test]
fn obsolete_setup_commands_are_not_supported() {
    for message in [
        serde_json::json!({
            "type": "job_facts",
            "job_id": fixtures::job_id(),
            "facts": fixtures::facts(),
        }),
        serde_json::json!({
            "type": "docker_disk",
            "job_id": fixtures::job_id(),
            "disk": {
                "free_bytes": 20_000_000_000_u64,
                "drive_id": "docker_cache",
                "access": "disposable_copy",
            },
        }),
    ] {
        let decoded = serde_json::from_value::<AgentToGuest>(message.clone());
        assert_eq!(
            decoded.map(|_| ()).map_err(|error| error.classify()),
            Err(serde_json::error::Category::Data),
            "obsolete command: {message}"
        );
    }
}

#[test]
fn shutdown_remains_a_tagged_unit_message() {
    snapshot("vsock_shutdown", "shutdown", AgentToGuest::Shutdown);
}

#[test]
fn guest_hello_reports_the_booted_image_and_protocol() {
    snapshot(
        "guest_hello",
        "hello",
        GuestToAgent::Hello {
            handoff_protocol_version: HANDOFF_PROTOCOL_VERSION,
            configuration_received: false,
            protocol_version: ProtocolVersion::CURRENT,
            image_id: fixtures::image_id(),
        },
    );
}

#[test]
fn guest_runner_ready_identifies_the_configured_runner() {
    snapshot(
        "guest_runner_ready",
        "runner_ready",
        GuestToAgent::RunnerReady {
            runner_id: fixtures::runner_id(),
        },
    );
}

#[test]
fn guest_runner_exited_reports_the_process_exit_code() {
    snapshot(
        "guest_runner_exited",
        "runner_exited",
        GuestToAgent::RunnerExited {
            runner_id: fixtures::runner_id(),
            exit: RunnerExit { code: Some(0) },
            usage: None,
        },
    );
}

#[test]
fn guest_runner_exit_round_trips_with_job_usage() {
    snapshot(
        "guest_runner_exited_with_usage",
        "runner_exited",
        GuestToAgent::RunnerExited {
            runner_id: fixtures::runner_id(),
            exit: RunnerExit { code: Some(0) },
            usage: Some(fixtures::job_usage()),
        },
    );
}

#[test]
fn boat_usage_samples_round_trip_with_a_step_still_running() {
    let mut usage = fixtures::job_usage();
    usage.steps.truncate(1);
    usage.steps[0].timing = JobStepTiming::Ran {
        started_at_ms: Some(1_750_000_000_000),
        ended_at_ms: None,
    };
    snapshot(
        "guest_usage_samples",
        "usage_samples",
        GuestToAgent::UsageSamples {
            runner_id: fixtures::runner_id(),
            usage,
        },
    );
}

#[test]
fn preparation_decisions_name_the_job_and_selected_cache_route() {
    snapshot(
        "vsock_job_github_only",
        "prepare_job",
        AgentToGuest::PrepareJob {
            job_id: fixtures::job_id(),
            decision: JobDecision::GitHubOnly,
        },
    );
    snapshot(
        "vsock_job_machine_cache",
        "prepare_job",
        AgentToGuest::PrepareJob {
            job_id: fixtures::job_id(),
            decision: JobDecision::MachineCache {
                facts: Box::new(fixtures::facts()),
                cache: fixtures::cache_connection(),
                docker: None,
            },
        },
    );
    snapshot(
        "vsock_job_machine_cache_docker",
        "prepare_job",
        AgentToGuest::PrepareJob {
            job_id: fixtures::job_id(),
            decision: JobDecision::MachineCache {
                facts: Box::new(fixtures::facts()),
                cache: fixtures::cache_connection(),
                docker: Some(DockerDiskAttachment {
                    free_bytes: 20_000_000_000,
                    drive_id: "docker_cache".into(),
                    access: DockerDiskAccess::DisposableCopy,
                }),
            },
        },
    );
}

#[test]
fn guest_hello_without_the_host_decision_format_is_not_supported() {
    let decoded = serde_json::from_value::<GuestToAgent>(serde_json::json!({
        "type": "hello", "protocol_version": 1, "image_id": "old-image",
        "configuration_received": true,
    }));
    assert_eq!(
        decoded.map_err(|error| error.classify()),
        Err(serde_json::error::Category::Data)
    );
}
