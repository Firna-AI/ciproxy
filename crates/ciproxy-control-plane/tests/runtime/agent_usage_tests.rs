//! Agent usage admission through the relay and the real store.

use std::sync::Arc;

use chrono::Duration;
use ciproxy_control_plane::hosted_events::Relay;
use ciproxy_control_plane::runtime::{MachineRuntime, OwnMachines};
use ciproxy_gateway::boundary::AgentEvents;
use ciproxy_gateway::types::Session;
use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::identifiers::RunnerId;
use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::runners::RunnerExit;
use ciproxy_protocol::usage::{JobStep, JobStepTiming, JobUsage};
use ciproxy_store_interface::job_usage::StoredJobUsage;
use ciproxy_store_interface::runners::RunnerState;
use unimock::Unimock;
use uuid::Uuid;

use super::clock::clock;
use super::recovery_support::{runtime_with_clock, saved};
use super::support::{Fixture, at};

fn job_usage() -> JobUsage {
    JobUsage {
        platform: Platform::LinuxX64,
        first_point_at_ms: (at() - Duration::seconds(1)).timestamp_millis(),
        point_seconds: 1,
        cpu_milli_vcpu: vec![1000],
        peak_memory_mib: vec![256],
        oom_kills: Some(vec![0]),
        steps: vec![JobStep {
            name: "Run make test".into(),
            timing: JobStepTiming::Ran {
                started_at_ms: Some((at() - Duration::seconds(1)).timestamp_millis()),
                ended_at_ms: Some(at().timestamp_millis()),
            },
        }],
    }
}

fn receive_exit(fixture: &Fixture, machine_id: Uuid, usage: JobUsage) -> Arc<MachineRuntime> {
    let runner_id = fixture.intent.runner.id;
    fixture
        .store
        .update_runner(
            fixture.workspace,
            fixture.organization,
            runner_id,
            RunnerState::Busy,
            Some(47),
        )
        .unwrap();
    let runtime = Arc::new(runtime_with_clock(
        fixture,
        Arc::new(Unimock::new(())),
        Arc::new(Unimock::new(())),
        clock(false),
    ));
    let boundary: Arc<dyn OwnMachines> = runtime.clone();
    let relay = Relay::new();
    relay.bind(&boundary).unwrap();
    relay
        .receive(
            Session {
                workspace_id: fixture.workspace,
                machine_id,
                id: fixture
                    .store
                    .machine_session(fixture.workspace, machine_id)
                    .unwrap()
                    .unwrap(),
            },
            AgentMessage::RunnerExited {
                runner_id: RunnerId(runner_id.to_string()),
                exit: RunnerExit { code: Some(0) },
                usage: Some(usage.into()),
            },
        )
        .unwrap();
    runtime
}

fn saved_usage(fixture: &Fixture) -> Option<StoredJobUsage> {
    fixture
        .store
        .job_usage(fixture.workspace, fixture.intent.runner.id)
        .unwrap()
}

#[tokio::test]
async fn gateway_saves_owned_usage_before_processing_the_exit() {
    let fixture = Fixture::new();
    let mut allocation = fixture.intent.runner.size;
    allocation.vcpu = 3;
    allocation.memory_bytes = 9_000_000_000;
    fixture
        .store
        .update_runner_size(
            fixture.workspace,
            fixture.organization,
            fixture.intent.runner.id,
            allocation,
        )
        .unwrap();
    let usage = JobUsage {
        cpu_milli_vcpu: vec![2750],
        peak_memory_mib: vec![8000],
        ..job_usage()
    };
    let runtime = receive_exit(&fixture, fixture.machine, usage.clone());
    assert_eq!(saved(&fixture).runner.state, RunnerState::Busy);
    let mut expected = usage;
    expected.steps[0].name = "Run script".into();
    assert_eq!(
        saved_usage(&fixture),
        Some(StoredJobUsage {
            usage: expected,
            is_final: true,
            received_at: at()
        })
    );
    runtime.quiesce().await.unwrap();
    assert_eq!(saved(&fixture).runner.state, RunnerState::Stopped);
    runtime.shutdown().await.unwrap();
}

#[tokio::test]
async fn gateway_does_not_save_usage_for_another_machines_runner() {
    let mut fixture = Fixture::new();
    let other_machine = fixture.add_machine();
    let runtime = receive_exit(&fixture, other_machine, job_usage());
    runtime.quiesce().await.unwrap();
    assert_eq!(saved(&fixture).runner.state, RunnerState::Busy);
    assert_eq!(saved_usage(&fixture), None);
    runtime.shutdown().await.unwrap();
}

#[tokio::test]
async fn gateway_keeps_the_exit_when_usage_exceeds_the_vm_allocation() {
    let fixture = Fixture::new();
    let usage = JobUsage {
        cpu_milli_vcpu: vec![2001],
        ..job_usage()
    };
    let runtime = receive_exit(&fixture, fixture.machine, usage);
    runtime.quiesce().await.unwrap();
    assert_eq!(saved(&fixture).runner.state, RunnerState::Stopped);
    assert_eq!(saved_usage(&fixture), None);
    runtime.shutdown().await.unwrap();
}
