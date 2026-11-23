//! Agent-originated JSON, including registration and reconnect state.

use crate::agent::{AgentMessage, CommandFailure, CommandOutcome};
use crate::cache::{DockerDiskReport, MachineCacheReport};
use crate::platform::{Platform, Virtualization};
use crate::reports::{DiskReport, HardwareReport, MachineReport};
use crate::resources::GIGABYTE;
use crate::runners::RunnerExit;
use crate::secret::Secret;
use crate::statistics::Statistics;
use crate::version::ProtocolVersion;

use super::assertions::snapshot;
use super::fixtures;

#[test]
fn legacy_machine_reports_keep_both_statistics_sections_measured() {
    let report: MachineReport = serde_json::from_value(serde_json::json!({
        "hostname":"legacy", "agent_version":"old",
        "hardware":{"platform":"linux_x64", "processor":"x64", "threads":0, "memory_bytes":0,
            "virtualization":"kvm_unavailable", "disks":[]},
        "capacity":0, "job_disk_bytes":0, "slots":[], "runners":[], "drain":"active",
        "cache":{"used_bytes":0, "capacity_bytes":0, "restores_last_7_days":0, "hits_last_7_days":0, "jobs":[]},
        "docker_disks":{"leased_bytes":0, "repositories":0, "used_bytes":0},
        "images":{"available":[], "newest":null}
    })).unwrap();
    assert_eq!(
        report.cache,
        Statistics::Measured(MachineCacheReport {
            used_bytes: 0,
            capacity_bytes: 0,
            restores_last_7_days: 0,
            hits_last_7_days: 0,
            unknown_jobs: vec![],
            jobs: vec![],
        })
    );
    assert_eq!(
        report.docker_disks,
        Statistics::Measured(DockerDiskReport {
            leased_bytes: 0,
            repositories: 0,
            used_bytes: 0,
        })
    );
}

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
fn mac_report_carries_its_platform_and_virtualization_state() {
    let mut report = fixtures::report();
    report.hardware = HardwareReport {
        platform: Platform::MacosArm64,
        processor: "Apple M4".into(),
        threads: 10,
        memory_bytes: 16 * GIGABYTE,
        virtualization: Virtualization::EntitlementMissing,
        disks: vec![DiskReport {
            name: "disk3s1".into(),
            capacity_bytes: 512 * GIGABYTE,
            free_bytes: 300 * GIGABYTE,
        }],
    };
    snapshot(
        "agent_report_mac",
        "report",
        AgentMessage::Report { report },
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
            usage: None,
        },
    );
}

#[test]
fn agent_runner_exit_round_trips_with_macos_usage() {
    let mut usage = fixtures::job_usage();
    usage.platform = Platform::MacosArm64;
    usage.oom_kills = None;
    snapshot(
        "agent_runner_exited_with_usage",
        "runner_exited",
        AgentMessage::RunnerExited {
            runner_id: fixtures::runner_id(),
            exit: RunnerExit { code: Some(0) },
            usage: Some(usage),
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
