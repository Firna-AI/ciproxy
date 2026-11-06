//! Missing App repository metadata still releases each accepted assignment.

use std::sync::{Arc, Mutex};

use ciproxy_control_plane::error;
use ciproxy_control_plane::runtime::OwnMachines;
use ciproxy_gateway::gateway::MachineGatewayMock;
use ciproxy_gateway::types::Delivery;
use ciproxy_github::assignments::JobAssignmentsMock;
use ciproxy_github::error::Error;
use ciproxy_github::runtime::RuntimeMock;
use ciproxy_protocol::agent::{AgentMessage, ControlMessage, JobPreparationOutcome};
use ciproxy_protocol::identifiers::{CommandId, JobId, OrganizationId, RunnerId};
use ciproxy_store_interface::github::Repository;
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

use super::recovery_support::{gateway, job, runtime, runtime_with_clock};
use super::support::{Fixture, at};

#[tokio::test]
async fn removed_repository_delivers_a_github_only_decision_on_job_and_tick() {
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
    let job = job(&fixture);
    fixture
        .store
        .save_github_job(fixture.workspace, &job, at())
        .unwrap();
    fixture
        .store
        .replace_repositories(fixture.workspace, fixture.organization, &[])
        .unwrap();
    let messages = Arc::new(Mutex::new(Vec::new()));
    let sent = messages.clone();
    let workspace = fixture.workspace;
    let machine = fixture.machine;
    let runtime = runtime(
        &fixture,
        Arc::new(Unimock::new(())),
        Arc::new(Unimock::new(
            MachineGatewayMock::send
                .each_call(matching!(_, _, _))
                .answers_arc(Arc::new(move |_, owner, destination, message| {
                    assert_eq!((owner, destination), (workspace, machine));
                    let budget = matches!(message, ControlMessage::DockerBudget { .. });
                    sent.lock().unwrap().push(message);
                    Ok(if budget {
                        Delivery::Offline
                    } else {
                        Delivery::Connected
                    })
                })),
        )),
    );
    runtime.job(fixture.workspace, job.clone()).await.unwrap();
    runtime.tick().await.unwrap();
    runtime
        .agent(
            fixture.workspace,
            fixture.machine,
            AgentMessage::JobStarted {
                runner_id: RunnerId(fixture.intent.runner.id.to_string()),
                job_id: JobId(job.id.to_string()),
            },
        )
        .await
        .unwrap();
    let decisions: Vec<_> = messages
        .lock()
        .unwrap()
        .iter()
        .filter(|message| !matches!(message, ControlMessage::DockerBudget { .. }))
        .cloned()
        .collect();
    assert_eq!(decisions, vec![github_only(&fixture, job.id); 3]);
}

fn github_only(fixture: &Fixture, job_id: Uuid) -> ControlMessage {
    ControlMessage::JobPreparation {
        command_id: CommandId(Uuid::from_u128(99).to_string()),
        runner_id: RunnerId(fixture.intent.runner.id.to_string()),
        job_id: JobId(job_id.to_string()),
        outcome: JobPreparationOutcome::GitHubOnly {
            organization_id: OrganizationId("42".into()),
        },
    }
}

#[tokio::test]
async fn transient_assignment_failure_returns_an_error_and_retries_without_fallback() {
    let fixture = Fixture::new();
    let job = job(&fixture);
    let messages = Arc::new(Mutex::new(Vec::new()));
    let mut runtime = runtime_with_clock(
        &fixture,
        Arc::new(Unimock::new(())),
        gateway(messages.clone()),
        Arc::new(Unimock::new(
            RuntimeMock::uuid
                .each_call(matching!())
                .returns(Uuid::from_u128(99)),
        )),
    );
    runtime.assignments = Arc::new(Unimock::new(JobAssignmentsMock::facts.stub(|each| {
        each.call(matching!(_, _, _))
            .answers(&|_, _, _, _| Err(Error::AccessUnavailable(503)))
            .once()
            .then()
            .answers(&|_, _, _, _| {
                Ok(JobPreparationOutcome::GitHubOnly {
                    organization_id: OrganizationId("42".into()),
                })
            });
    })));
    let failed = runtime.job(fixture.workspace, job.clone()).await;
    assert!(matches!(failed, Err(error::Error::Internal(_))));
    let error = failed.unwrap_err();
    assert!(matches!(
        error,
        error::Error::Internal(ref internal)
            if matches!(internal.source_ref().downcast_ref::<Error>(), Some(Error::AccessUnavailable(503)))
    ));
    assert_eq!(*messages.lock().unwrap(), Vec::new());
    runtime.job(fixture.workspace, job.clone()).await.unwrap();
    assert_eq!(
        *messages.lock().unwrap(),
        vec![github_only(&fixture, job.id)]
    );
}
