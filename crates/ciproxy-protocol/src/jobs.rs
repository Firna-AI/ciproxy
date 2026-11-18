//! Job identity and cache-scope inputs, without a source or trust decision.

use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};

use crate::identifiers::{GitRef, JobEvent, JobId, OrganizationId, RepositoryId};

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
    /// Expected token identities from the bound Worker, scale set, and JIT runner.
    pub assignment: JobAssignment,
    /// The event that caused the job.
    pub event: JobEvent,
    /// Head repository when the job has one, including a pull request fork.
    pub head_repository: Option<Repository>,
    /// Default branch ref used by cache restores.
    pub default_ref: GitRef,
    /// Whether this workspace's confirmed tier includes sticky Docker disks.
    pub sticky_docker: bool,
}

/// Job UUIDs of the exact one-job runner registered with GitHub.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct JobIdBinding {
    /// Job UUID reported by the runner Worker and used for guest handoff.
    pub worker_job_id: JobId,
    /// Job UUID from the hosted scale-set assignment for the same runner.
    pub scale_set_job_id: JobId,
}

/// GitHub identities expected in the runtime token, never inferred from a ref.
///
/// Legacy non-null job IDs read as equal pairs. Serialization emits the pair.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct JobAssignment {
    /// Workflow run ID from the scale-set message.
    pub run_id: String,
    /// Both job UUIDs bound to the exact registered runner.
    pub job_ids: JobIdBinding,
    /// GitHub runner ID from the JIT registration, not ciproxy's runner ID.
    pub runner_id: String,
}

#[derive(Deserialize)]
struct JobAssignmentInput {
    run_id: String,
    job_ids: Option<JobIdBinding>,
    job_id: Option<JobId>,
    runner_id: String,
}

impl<'de> Deserialize<'de> for JobAssignment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let input = JobAssignmentInput::deserialize(deserializer)?;
        let job_ids = match (input.job_ids, input.job_id) {
            (Some(job_ids), _) => job_ids,
            (None, Some(job_id)) => JobIdBinding {
                worker_job_id: job_id.clone(),
                scale_set_job_id: job_id,
            },
            (None, None) => return Err(D::Error::missing_field("job_ids")),
        };
        Ok(Self {
            run_id: input.run_id,
            job_ids,
            runner_id: input.runner_id,
        })
    }
}

/// Scope checked by the machine, returned only through its admin interface.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum JobScope {
    /// Exactly one signed access-control entry has the write bit.
    Checked {
        /// GitHub's full scope key, with no tag normalization.
        git_ref: GitRef,
        /// Readable full scope keys from the same signed token.
        read_refs: Vec<GitRef>,
    },
    /// No ref can be trusted. Only default-branch local restores are allowed.
    Unknown {
        /// Safe diagnostic that contains no token or signature contents.
        reason: UnknownJobScopeReason,
    },
}

/// Reasons that a runtime token cannot provide a checked job scope.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnknownJobScopeReason {
    /// No original runtime token was supplied.
    MissingToken,
    /// The token or required claim has an unreadable format.
    MalformedToken,
    /// The token does not use RS256.
    UnsupportedAlgorithm,
    /// The token has no signing-key identifier.
    MissingKeyId,
    /// GitHub's public keys could not be fetched.
    KeysUnavailable,
    /// No fetched public key has this identifier.
    UnknownKey,
    /// The signature is missing or does not verify.
    InvalidSignature,
    /// The issuer does not exactly match GitHub Actions.
    InvalidIssuer,
    /// The validity interval is reversed.
    InvalidValidity,
    /// The token is not active yet.
    NotYetValid,
    /// The token has expired.
    Expired,
    /// The token belongs to another repository.
    RepositoryMismatch,
    /// The token belongs to another workflow run.
    RunMismatch,
    /// The token belongs to another registered runner.
    RunnerMismatch,
    /// The token has neither job UUID of the registered runner's binding.
    JobMismatch,
    /// The access-control claim is not an encoded array of scopes.
    InvalidAccessControl,
    /// No signed scope has the write bit.
    NoWriteScope,
    /// More than one signed scope has the write bit.
    MultipleWriteScopes,
}
