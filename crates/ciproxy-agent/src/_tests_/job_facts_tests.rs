//! Only matching authenticated job facts cross into guest or cache configuration.

use std::sync::Arc;

use ciproxy_protocol::agent::{AgentMessage, CommandFailure, CommandOutcome, ControlMessage};
use ciproxy_protocol::guest::AgentToGuest;
use ciproxy_protocol::identifiers::{
    CommandId, GitRef, JobEvent, JobId, OrganizationId, RepositoryId, RunnerId,
};
use ciproxy_protocol::jobs::{JobAssignment, JobFacts, Repository};
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{CacheMock, VmsMock};
use crate::controller::Agent;
use crate::fixtures::{applied, controller, vm};
use crate::state::MachineState;

#[tokio::test]
async fn authenticated_facts_must_match_the_runner_organization_and_observed_job() {
    let facts = JobFacts {
        organization_id: OrganizationId("org-1".to_owned()),
        repository: Repository {
            id: RepositoryId("repo-1".to_owned()),
            full_name: "first/project".to_owned(),
        },
        assignment: JobAssignment {
            run_id: "run-1".to_owned(),
            job_id: Some(JobId("job-1".to_owned())),
            runner_id: "github-runner-1".to_owned(),
        },
        event: JobEvent("pull_request".to_owned()),
        head_repository: None,
        default_ref: GitRef("refs/heads/main".to_owned()),
        sticky_docker: false,
    };
    let cache_facts = facts.clone();
    let guest_message = AgentToGuest::PrepareJob {
        cache: None,
        docker: None,
        job_id: JobId("job-1".to_owned()),
        facts: facts.clone(),
    };
    let mock = Unimock::new((
        crate::boundaries::StoreMock::save
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
        CacheMock::prepare
            .some_call(matching!((runner, _) if runner.0 == "job"))
            .answers_arc(Arc::new(move |_, _, facts| {
                assert_eq!(facts, &cache_facts);
                Ok(None)
            })),
        VmsMock::send
            .some_call(matching!((record, _) if record.report.runner_id.0 == "job"))
            .answers_arc(Arc::new(move |_, _, message| {
                assert_eq!(message, &guest_message);
                Ok(())
            })),
    ));
    let agent = controller(
        MachineState {
            runners: vec![vm("job", true)],
            ..MachineState::default()
        },
        mock,
    );
    let rejected = Some(AgentMessage::CommandResult {
        command_id: CommandId("command-1".to_owned()),
        outcome: CommandOutcome::Failed {
            reason: CommandFailure::GuestConnection,
        },
    });
    let mut wrong_organization = facts.clone();
    wrong_organization.organization_id = OrganizationId("org-2".to_owned());
    for (job_id, facts) in [
        (JobId("job-2".to_owned()), facts.clone()),
        (JobId("job-1".to_owned()), wrong_organization),
    ] {
        assert_eq!(
            agent
                .control(ControlMessage::JobFacts {
                    command_id: CommandId("command-1".to_owned()),
                    runner_id: RunnerId("job".to_owned()),
                    job_id,
                    facts
                })
                .await
                .unwrap(),
            rejected
        );
    }
    assert_eq!(
        agent
            .control(ControlMessage::JobFacts {
                command_id: CommandId("command-1".to_owned()),
                runner_id: RunnerId("job".to_owned()),
                job_id: JobId("job-1".to_owned()),
                facts
            })
            .await
            .unwrap(),
        applied()
    );
}

#[tokio::test]
async fn trusted_runtime_credentials_stay_on_the_machine_and_out_of_reports() {
    let runtime = ciproxy_protocol::runtime::RuntimeCredentials {
        docker_data_bytes: 0,
        proxy_url: "http://172.21.0.1:8230/".to_owned(),
        github_token: ciproxy_protocol::secret::Secret::new("original-runtime-credential"),
        cache_url: ciproxy_protocol::secret::Secret::new("https://cache.example/"),
        results_url: ciproxy_protocol::secret::Secret::new("https://results.example/"),
        permissions: ciproxy_protocol::runtime::CachePermissions {
            read: true,
            write: false,
        },
    };
    let expected = runtime.clone();
    let mock = Unimock::new((
        CacheMock::handoff
            .some_call(matching!(_, _, _))
            .answers_arc(Arc::new(move |_, vm, job, runtime| {
                assert_eq!(vm.report.runner_id, RunnerId("job".to_owned()));
                assert_eq!(job, &JobId("job-1".to_owned()));
                assert_eq!(runtime, expected);
                Ok(())
            })),
        crate::boundaries::StoreMock::save
            .some_call(matching!(_))
            .answers(&|_, state| {
                assert!(
                    !serde_json::to_string(state)
                        .unwrap()
                        .contains("original-runtime-credential")
                );
                Ok(())
            }),
    ));
    let agent = controller(
        MachineState {
            runners: vec![vm("job", false)],
            ..MachineState::default()
        },
        mock,
    );
    let response = agent
        .guest(
            "job",
            ciproxy_protocol::guest::GuestToAgent::JobStarting {
                runner_id: RunnerId("job".to_owned()),
                job_id: JobId("job-1".to_owned()),
                runtime,
            },
        )
        .await
        .unwrap();
    assert_eq!(
        response,
        Some(AgentMessage::JobStarted {
            runner_id: RunnerId("job".to_owned()),
            job_id: JobId("job-1".to_owned())
        })
    );
    assert!(
        !serde_json::to_string(&response)
            .unwrap()
            .contains("original-runtime-credential")
    );
}
