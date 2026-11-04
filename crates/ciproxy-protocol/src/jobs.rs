//! Job identity and cache-scope inputs, without a source or trust decision.

use serde::{Deserialize, Serialize};

use crate::identifiers::{GitRef, JobEvent, OrganizationId, RepositoryId};

/// A repository identity supplied with the facts of a job.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Repository {
    /// GitHub repository identifier, distinct from its changeable name.
    pub id: RepositoryId,
    /// Owner and repository name, separated by a slash.
    pub full_name: String,
}

/// Facts needed to identify a job and scope its cache and Docker disk.
///
/// These are values only. The caller must obtain them from an approved source.
/// Their presence does not grant shared-cache or Docker-disk write access.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct JobFacts {
    /// GitHub organization to which the assigned job belongs.
    pub organization_id: OrganizationId,
    /// Repository to which the job belongs.
    pub repository: Repository,
    /// The job's ref, rather than the ref of a reusable workflow file.
    pub git_ref: GitRef,
    /// The event that caused the job.
    pub event: JobEvent,
    /// Head repository when the job has one, including a pull request fork.
    pub head_repository: Option<Repository>,
    /// Default branch ref used by cache restores.
    pub default_ref: GitRef,
    /// Pull request base branch ref, absent for jobs without a base branch.
    pub base_ref: Option<GitRef>,
}
