//! Expected runtime-token identities assembled from trusted hosted data.

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_protocol::identifiers::{GitRef, JobEvent, JobId, OrganizationId, RepositoryId};
use ciproxy_protocol::jobs::{JobAssignment, JobFacts, Repository};
use ciproxy_store_interface::github::GithubJob;

use crate::error::Result;
use crate::persistence::assignment::AssignmentStore;
use crate::persistence::session::ListenerTarget;
use crate::scale_sets::JitRegistration;

/// Assignment facts for the agent, with no inferred or trusted job ref.
#[async_trait]
pub trait JobAssignments: Send + Sync {
    /// Build facts from a saved scale-message job, repository snapshot, and JIT.
    ///
    /// None means metadata is unavailable. The job can use GitHub directly.
    async fn facts(
        &self,
        target: ListenerTarget,
        job: &GithubJob,
        jit: &JitRegistration,
    ) -> Result<Option<JobFacts>>;
}

/// Hosted assembly boundary consumed through dyn JobAssignments.
pub struct AssignmentService {
    /// Workspace-scoped repository metadata from the trusted API snapshot.
    pub store: Arc<dyn AssignmentStore>,
}

#[async_trait]
impl JobAssignments for AssignmentService {
    async fn facts(
        &self,
        target: ListenerTarget,
        job: &GithubJob,
        jit: &JitRegistration,
    ) -> Result<Option<JobFacts>> {
        let snapshot = self.store.snapshot(target).await?;
        let Some(repository) = snapshot
            .repositories
            .iter()
            .find(|repo| repo.id == job.facts.repository.id)
        else {
            return Ok(None);
        };
        Ok(Some(JobFacts {
            organization_id: OrganizationId(snapshot.organization_id.to_string()),
            repository: Repository {
                id: RepositoryId(job.facts.repository.id.to_string()),
                full_name: job.facts.repository.full_name.clone(),
            },
            assignment: JobAssignment {
                run_id: job.run_id.to_string(),
                job_id: Some(JobId(job.id.to_string())),
                runner_id: jit.runner_id.to_string(),
            },
            event: JobEvent(job.facts.event.clone()),
            head_repository: job.facts.head_repository.as_ref().map(|head| Repository {
                id: RepositoryId(head.id.to_string()),
                full_name: head.full_name.clone(),
            }),
            default_ref: GitRef(format!("refs/heads/{}", repository.default_branch)),
        }))
    }
}
