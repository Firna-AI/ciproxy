//! Authenticated job observations keep an immutable identity for the exact runner.

use std::sync::{Arc, Mutex};

use ciproxy_control_plane::error::Error;
use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::identifiers::{JobId, RunnerId};
use ciproxy_protocol::runners;
use ciproxy_store_interface::job_bindings::WorkerJobId;
use ciproxy_store_interface::runners::{RunnerPlace, RunnerState};
use unimock::Unimock;
use uuid::Uuid;

use super::clock::clock;
use super::recovery_support::{gateway, job, runtime, runtime_with_clock, saved};
use super::settled::Settled;
use super::support::{Fixture, at};

#[tokio::test]
async fn job_started_binds_the_worker_once_and_keeps_it_after_runner_exit() {
    let fixture = Fixture::new();
    let runtime = runtime_with_clock(
        &fixture,
        Arc::new(Unimock::new(())),
        Arc::new(Unimock::new(())),
        clock(false),
    );
    let worker = JobId(Uuid::from_u128(0x41d84499).to_string());
    for job_id in [
        worker.clone(),
        worker.clone(),
        JobId("conflicting-worker".into()),
    ] {
        runtime
            .settled_agent(
                fixture.workspace,
                fixture.machine,
                AgentMessage::JobStarted {
                    runner_id: RunnerId(fixture.intent.runner.id.to_string()),
                    job_id,
                },
            )
            .await
            .unwrap();
    }
    assert_eq!(
        saved(&fixture).worker_job_id,
        Some(WorkerJobId(worker.clone()))
    );
    runtime
        .settled_agent(
            fixture.workspace,
            fixture.machine,
            AgentMessage::RunnerExited {
                runner_id: RunnerId(fixture.intent.runner.id.to_string()),
                exit: runners::RunnerExit { code: Some(0) },
            },
        )
        .await
        .unwrap();
    let ended = saved(&fixture);
    assert_eq!(ended.runner.state, RunnerState::Stopped);
    assert_eq!(ended.worker_job_id, Some(WorkerJobId(worker)));
}

#[tokio::test]
async fn another_machine_or_runner_report_never_binds_the_registered_runner() {
    let fixture = Fixture::new();
    let runtime = runtime_with_clock(
        &fixture,
        Arc::new(Unimock::new(())),
        Arc::new(Unimock::new(())),
        Arc::new(Unimock::new(())),
    );
    for (machine_id, runner_id) in [
        (Uuid::new_v4(), fixture.intent.runner.id),
        (fixture.machine, Uuid::new_v4()),
    ] {
        let result = runtime
            .settled_agent(
                fixture.workspace,
                machine_id,
                AgentMessage::JobStarted {
                    runner_id: RunnerId(runner_id.to_string()),
                    job_id: JobId("other-worker".into()),
                },
            )
            .await;
        assert!(matches!(result, Err(Error::RunnerOwnership)));
        assert_eq!(saved(&fixture).worker_job_id, None);
    }
    let assignment = job(&fixture);
    fixture
        .store
        .save_github_job(fixture.workspace, &assignment, at())
        .unwrap();
    let mut other = fixture.intent.clone();
    other.runner.id = Uuid::new_v4();
    other.runner.github_id = Some(48);
    other.runner.place = RunnerPlace::Machine {
        machine_id: fixture.machine,
        first_slot: 1,
    };
    other.slot_indices = vec![1];
    other.command_id = Uuid::new_v4();
    fixture
        .store
        .create_own_runner(fixture.workspace, &other)
        .unwrap();
    let observed_runtime = runtime_with_clock(
        &fixture,
        Arc::new(Unimock::new(())),
        Arc::new(Unimock::new(())),
        clock(false),
    );
    observed_runtime
        .settled_agent(
            fixture.workspace,
            fixture.machine,
            AgentMessage::JobStarted {
                runner_id: RunnerId(other.runner.id.to_string()),
                job_id: JobId(assignment.id.to_string()),
            },
        )
        .await
        .unwrap();
    let original = fixture
        .store
        .own_runners(fixture.workspace)
        .unwrap()
        .into_iter()
        .find(|runner| runner.runner.id == fixture.intent.runner.id)
        .unwrap();
    assert_eq!(original.worker_job_id, None);
}

#[tokio::test]
async fn full_running_report_recovers_a_missing_start_event_after_runtime_restart() {
    let mut fixture = Fixture::new();
    let worker = JobId(Uuid::from_u128(0x6bb53774).to_string());
    fixture.contact.report.runners[0].state = runners::RunnerState::Running {
        job_id: worker.clone(),
    };
    fixture
        .store
        .record_contact(fixture.workspace, fixture.machine, &fixture.contact)
        .unwrap();
    let runtime = runtime(
        &fixture,
        Arc::new(Unimock::new(())),
        gateway(Arc::new(Mutex::new(Vec::new()))),
    );
    runtime.settled_tick().await.unwrap();
    assert_eq!(saved(&fixture).worker_job_id, Some(WorkerJobId(worker)));
    assert_eq!(saved(&fixture).runner.state, RunnerState::Busy);
}
