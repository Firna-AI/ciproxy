//! Both directions of the private agent-to-guest vsock protocol.

use crate::cache::{DockerDiskAccess, DockerDiskAttachment};
use crate::guest::HANDOFF_PROTOCOL_VERSION;
use crate::guest::{AgentToGuest, GuestToAgent, JobDecision};
use crate::runners::RunnerExit;
use crate::secret::Secret;
use crate::version::ProtocolVersion;

use super::assertions::snapshot;
use super::fixtures;

#[test]
fn configure_runner_delivers_registration_and_cache_access_over_vsock() {
    snapshot(
        "vsock_configure_runner",
        "configure_runner",
        AgentToGuest::ConfigureRunner {
            protocol_version: ProtocolVersion::CURRENT,
            runner_id: fixtures::runner_id(),
            organization_id: fixtures::organization_id(),
            registration: Secret::new(fixtures::REGISTRATION),
            cache: Some(fixtures::cache_connection()),
        },
    );
}

#[test]
fn guest_job_facts_preserve_all_cache_scope_inputs() {
    snapshot(
        "vsock_job_facts",
        "job_facts",
        AgentToGuest::JobFacts {
            job_id: fixtures::job_id(),
            facts: fixtures::facts(),
        },
    );
}

#[test]
fn docker_disk_identifies_the_job_drive_and_copy_mode() {
    snapshot(
        "vsock_docker_disk",
        "docker_disk",
        AgentToGuest::DockerDisk {
            job_id: fixtures::job_id(),
            disk: DockerDiskAttachment {
                free_bytes: 20_000_000_000,
                drive_id: "docker_cache".into(),
                access: DockerDiskAccess::DisposableCopy,
            },
        },
    );
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
fn guest_job_started_names_the_assignment_without_scope_claims() {
    snapshot(
        "guest_job_started",
        "job_started",
        GuestToAgent::JobStarted {
            runner_id: fixtures::runner_id(),
            job_id: fixtures::job_id(),
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
