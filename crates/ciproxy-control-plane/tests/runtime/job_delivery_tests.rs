//! A concurrent machine removal cannot abort trusted job-facts delivery progress.

use std::sync::{Arc, Mutex};

use ciproxy_control_plane::runtime::{MachineRuntime, OwnMachines};
use ciproxy_gateway::gateway::MachineGatewayMock;
use ciproxy_gateway::types::Delivery;
use ciproxy_github::assignments::AssignmentService;
use ciproxy_github::persistence::StoreAdapter;
use ciproxy_github::registrations::{RegistrationRetirement, RunnerRegistrationsMock};
use ciproxy_github::runtime::RuntimeMock;
use ciproxy_protocol::agent::ControlMessage;
use ciproxy_store_interface::github::{
    DemandPhase, GithubJob, Repository, RepositoryIdentity, RunFacts,
};
use ciproxy_store_interface::runners::RunnerState;
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

use super::support::{Fixture, at};

#[tokio::test]
async fn removal_between_facts_and_delivery_preserves_job_until_trusted_completion() {
    let fixture = Fixture::new();
    fixture
        .store
        .replace_repositories(
            fixture.workspace,
            fixture.organization,
            &[Repository {
                id: 8,
                full_name: "org/repo".to_owned(),
                private: true,
                default_branch: "main".to_owned(),
            }],
        )
        .unwrap();
    let mut job = GithubJob {
        id: Uuid::new_v4(),
        organization_id: fixture.organization,
        scale_set_id: fixture.intent.runner.scale_set_id,
        label_id: fixture.intent.runner.label_id,
        actions_job_id: None,
        request_id: 0,
        run_id: 7,
        repository: "org/repo".to_owned(),
        name: "build".to_owned(),
        workflow_ref: "ci.yml".to_owned(),
        phase: DemandPhase::Started,
        result: None,
        assigned_at: Some(at()),
        started_at: Some(at()),
        completed_at: None,
        runner_github_id: Some(47),
        facts: RunFacts {
            id: 7,
            repository: RepositoryIdentity {
                id: 8,
                full_name: "org/repo".to_owned(),
            },
            event: "push".to_owned(),
            head_branch: Some("main".to_owned()),
            head_sha: "fixture".to_owned(),
            head_repository: None,
        },
    };
    fixture
        .store
        .save_github_job(fixture.workspace, &job, at())
        .unwrap();
    let store = fixture.store.clone();
    let workspace = fixture.workspace;
    let machine = fixture.machine;
    let events = Arc::new(Mutex::new(Vec::new()));
    let delivered = events.clone();
    let gateway = Arc::new(Unimock::new(
        MachineGatewayMock::send
            .each_call(matching!(_, _, _))
            .answers_arc(Arc::new(move |_, _, _, message| {
                if let ControlMessage::DockerBudget { budget, .. } = message {
                    assert_eq!(budget.unwrap().image.image_id.0, "fixture");
                    delivered.lock().unwrap().push("budget");
                    return Ok(Delivery::Connected);
                }
                assert!(matches!(message, ControlMessage::JobPreparation { .. }));
                delivered.lock().unwrap().push("facts");
                store.remove_machine(workspace, machine, at()).unwrap();
                Err(ciproxy_gateway::error::Error::NotFound)
            })),
    ));
    let clock = Arc::new(Unimock::new((
        RuntimeMock::now.each_call(matching!()).returns(at()),
        RuntimeMock::uuid
            .each_call(matching!())
            .returns(Uuid::from_u128(99)),
    )));
    let runtime = MachineRuntime::new(
        fixture.store.clone(),
        gateway,
        Arc::new(Unimock::new(
            RunnerRegistrationsMock::retire
                .some_call(matching!(_, _, Some(47)))
                .returns(Ok(RegistrationRetirement::Removed { id: 47 })),
        )),
        Arc::new(AssignmentService {
            store: Arc::new(StoreAdapter::new(fixture.store.clone(), clock.clone())),
        }),
        clock,
        None,
        Arc::new(Unimock::new(())),
    );
    assert_eq!(
        runtime
            .job(fixture.workspace, job.clone())
            .await
            .map_err(|_| "workspace aborted"),
        Ok(())
    );
    assert_eq!(
        fixture.store.own_runners(fixture.workspace).unwrap()[0]
            .runner
            .state,
        RunnerState::Busy
    );
    assert_eq!(*events.lock().unwrap(), vec!["budget", "facts"]);
    job.phase = DemandPhase::Completed;
    fixture
        .store
        .save_github_job(fixture.workspace, &job, at())
        .unwrap();
    runtime.job(fixture.workspace, job).await.unwrap();
    assert_eq!(
        fixture.store.own_runners(fixture.workspace).unwrap()[0]
            .runner
            .state,
        RunnerState::Stopped
    );
}
