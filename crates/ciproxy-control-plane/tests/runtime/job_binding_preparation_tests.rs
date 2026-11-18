//! Different ID namespaces prepare through one exact registered runner binding.

use std::sync::{Arc, Mutex};

use ciproxy_github::registrations::{RegistrationRetirement, RunnerRegistrationsMock};
use ciproxy_protocol::agent::{AgentMessage, ControlMessage, JobPreparationOutcome};
use ciproxy_protocol::identifiers::{JobId, RunnerId};
use ciproxy_protocol::jobs::JobIdBinding;
use ciproxy_store_interface::github::{DemandPhase, Repository};
use ciproxy_store_interface::job_bindings::WorkerJobId;
use ciproxy_store_interface::runners::RunnerState;
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

use super::recovery_support::{gateway, job, runtime, saved};
use super::settled::Settled;
use super::support::{Fixture, at};

#[tokio::test]
async fn different_worker_id_gets_preparation_facts_and_completion_cleanup() {
    let fixture = Fixture::new();
    fixture
        .store
        .replace_repositories(
            fixture.workspace,
            fixture.organization,
            &[Repository {
                id: 8,
                full_name: "org/repo".into(),
                private: true,
                default_branch: "main".into(),
            }],
        )
        .unwrap();
    let mut assignment = job(&fixture);
    assignment.id = Uuid::from_u128(0x1e1c0aee);
    let worker = JobId(Uuid::from_u128(0x41d84499).to_string());
    fixture
        .store
        .save_github_job(fixture.workspace, &assignment, at())
        .unwrap();
    let mut unrelated = assignment.clone();
    unrelated.id = worker.0.parse().unwrap();
    unrelated.runner_github_id = None;
    unrelated.phase = DemandPhase::Completed;
    unrelated.started_at = None;
    fixture
        .store
        .save_github_job(fixture.workspace, &unrelated, at())
        .unwrap();
    let messages = Arc::new(Mutex::new(Vec::new()));
    let runtime = runtime(
        &fixture,
        Arc::new(Unimock::new(
            RunnerRegistrationsMock::retire
                .some_call(matching!(_, _, Some(47)))
                .returns(Ok(RegistrationRetirement::Removed { id: 47 })),
        )),
        gateway(messages.clone()),
    );
    runtime
        .settled_job(fixture.workspace, assignment.clone())
        .await
        .unwrap();
    assert_eq!(
        messages.lock().unwrap().len(),
        0,
        "preparation waits for its Worker binding"
    );
    runtime
        .settled_agent(
            fixture.workspace,
            fixture.machine,
            AgentMessage::JobStarted {
                runner_id: RunnerId(fixture.intent.runner.id.to_string()),
                job_id: worker.clone(),
            },
        )
        .await
        .unwrap();
    let preparation = messages
        .lock()
        .unwrap()
        .iter()
        .find(|message| matches!(message, ControlMessage::JobPreparation { .. }))
        .cloned()
        .unwrap();
    let ControlMessage::JobPreparation {
        runner_id,
        job_id,
        outcome,
        ..
    } = preparation
    else {
        unreachable!()
    };
    assert_eq!(runner_id, RunnerId(fixture.intent.runner.id.to_string()));
    assert_eq!(job_id, JobId("00000000-0000-0000-0000-000041d84499".into()));
    let JobPreparationOutcome::TrustedFacts { facts } = outcome else {
        panic!("trusted repository facts required")
    };
    assert_eq!(
        facts.assignment.job_ids,
        JobIdBinding {
            worker_job_id: JobId("00000000-0000-0000-0000-000041d84499".into()),
            scale_set_job_id: JobId("00000000-0000-0000-0000-00001e1c0aee".into()),
        }
    );
    assert_eq!(facts.assignment.runner_id, "47");
    fixture
        .store
        .remove_machine(fixture.workspace, fixture.machine, at())
        .unwrap();
    assignment.phase = DemandPhase::Completed;
    fixture
        .store
        .save_github_job(fixture.workspace, &assignment, at())
        .unwrap();
    runtime
        .settled_job(fixture.workspace, assignment)
        .await
        .unwrap();
    assert_eq!(saved(&fixture).runner.state, RunnerState::Stopped);
    assert_eq!(saved(&fixture).worker_job_id, Some(WorkerJobId(worker)));
}
