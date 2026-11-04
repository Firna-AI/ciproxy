//! Only matching authenticated job facts cross into guest or cache configuration.

use ciproxy_protocol::agent::{AgentMessage, CommandFailure, CommandOutcome, ControlMessage};
use ciproxy_protocol::guest::AgentToGuest;
use ciproxy_protocol::identifiers::{
    CommandId, GitRef, JobEvent, JobId, OrganizationId, RepositoryId, RunnerId,
};
use ciproxy_protocol::jobs::{JobFacts, Repository};
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
        git_ref: GitRef("refs/pull/8/merge".to_owned()),
        event: JobEvent("pull_request".to_owned()),
        head_repository: None,
        default_ref: GitRef("refs/heads/main".to_owned()),
        base_ref: Some(GitRef("refs/heads/main".to_owned())),
    };
    let mock = Unimock::new((
        CacheMock::facts.some_call(matching!((runner, facts) if runner.0 == "job" && facts.git_ref.0 == "refs/pull/8/merge")).returns(Ok(())),
        VmsMock::send.some_call(matching!((record, message) if record.report.runner_id.0 == "job"
            && matches!(message, AgentToGuest::JobFacts { facts, job_id } if facts.repository.id.0 == "repo-1" && job_id.0 == "job-1"))).returns(Ok(())),
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
