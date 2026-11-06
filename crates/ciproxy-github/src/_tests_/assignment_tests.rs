//! Public assignment assembly preserves API facts and message/JIT identities.

use std::sync::Arc;

use ciproxy_protocol::identifiers::{GitRef, JobEvent, JobId, OrganizationId, RepositoryId};
use ciproxy_protocol::jobs::{JobAssignment, JobFacts, Repository};
use ciproxy_store_interface::github::{DemandPhase, GithubJob, RepositoryIdentity};
use secrecy::SecretString;
use unimock::{MockFn, Unimock, matching};

use crate::assignments::{AssignmentService, JobAssignments};
use crate::persistence::assignment::{AssignmentSnapshot, AssignmentStoreMock};
use crate::scale_sets::JitRegistration;

use super::support::{facts, id, repositories, target};

fn job() -> GithubJob {
    GithubJob {
        id: id(0x71),
        organization_id: id(5),
        scale_set_id: id(6),
        label_id: id(2),
        actions_job_id: Some(999),
        request_id: 0,
        run_id: 31,
        repository: "customer/private".into(),
        name: "test".into(),
        workflow_ref: "shared/reusable/.github/workflows/test.yml@refs/tags/wrong".into(),
        phase: DemandPhase::Assigned,
        result: None,
        assigned_at: None,
        started_at: None,
        completed_at: None,
        runner_github_id: Some(888),
        facts: facts(),
    }
}

#[tokio::test]
async fn assignment_uses_api_repository_event_head_and_message_jit_ids() {
    let service: Box<dyn JobAssignments> = Box::new(AssignmentService {
        store: Arc::new(Unimock::new(
            AssignmentStoreMock::snapshot
                .some_call(matching!(_))
                .answers(&|_, received| {
                    assert_eq!(received, target());
                    Ok(AssignmentSnapshot {
                        sticky_docker: false,
                        organization_id: 17,
                        repositories: repositories(),
                    })
                }),
        )),
    });
    let mut job = job();
    job.facts.head_repository = Some(RepositoryIdentity {
        id: 29,
        full_name: "fork/project".into(),
    });
    let jit = JitRegistration {
        runner_id: 71,
        configuration: SecretString::from("secret-jit"),
    };
    assert_eq!(
        service.facts(target(), &job, &jit).await.unwrap(),
        Some(JobFacts {
            organization_id: OrganizationId("17".into()),
            repository: Repository {
                id: RepositoryId("23".into()),
                full_name: "customer/private".into()
            },
            assignment: JobAssignment {
                run_id: "31".into(),
                job_id: Some(JobId("00000000-0000-0000-0000-000000000071".into())),
                runner_id: "71".into()
            },
            event: JobEvent("pull_request_target".into()),
            head_repository: Some(Repository {
                id: RepositoryId("29".into()),
                full_name: "fork/project".into()
            }),
            default_ref: GitRef("refs/heads/trunk".into()),
            sticky_docker: false,
        })
    );
}

#[tokio::test]
async fn absent_repository_snapshot_never_guesses_a_default_branch() {
    let service: Box<dyn JobAssignments> = Box::new(AssignmentService {
        store: Arc::new(Unimock::new(
            AssignmentStoreMock::snapshot
                .some_call(matching!(_))
                .returns(Ok(AssignmentSnapshot {
                    sticky_docker: false,
                    organization_id: 17,
                    repositories: vec![],
                })),
        )),
    });
    let jit = JitRegistration {
        runner_id: 71,
        configuration: SecretString::from("secret-jit"),
    };
    assert_eq!(service.facts(target(), &job(), &jit).await.unwrap(), None);
}
