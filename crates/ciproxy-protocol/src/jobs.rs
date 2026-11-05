//! Job identity and cache-scope inputs, without a source or trust decision.

use serde::{Deserialize, Serialize};

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
    /// Expected token identity from the scale message and JIT registration.
    pub assignment: JobAssignment,
    /// The event that caused the job.
    pub event: JobEvent,
    /// Head repository when the job has one, including a pull request fork.
    pub head_repository: Option<Repository>,
    /// Default branch ref used by cache restores.
    pub default_ref: GitRef,
}

/// GitHub identities expected in the runtime token, never inferred from a ref.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct JobAssignment {
    /// Workflow run ID from the scale-set message.
    pub run_id: String,
    /// Scale-message job UUID, when the assignment provides it.
    pub job_id: Option<JobId>,
    /// GitHub runner ID from the JIT registration, not ciproxy's runner ID.
    pub runner_id: String,
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
    /// The token belongs to another scale-message job.
    JobMismatch,
    /// The access-control claim is not an encoded array of scopes.
    InvalidAccessControl,
    /// No signed scope has the write bit.
    NoWriteScope,
    /// More than one signed scope has the write bit.
    MultipleWriteScopes,
}
