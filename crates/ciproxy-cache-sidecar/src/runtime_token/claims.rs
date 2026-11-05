//! Typed token claims and the exact scope keys signed by GitHub.

use ciproxy_protocol::identifiers::GitRef;
use ciproxy_protocol::jobs::{JobFacts, JobScope, UnknownJobScopeReason};
use serde::Deserialize;
use serde::de::IgnoredAny;

pub(super) const ISSUER: &str = "https://token.actions.githubusercontent.com";
const LEEWAY: u64 = 30;

#[derive(Deserialize)]
#[serde(untagged)]
pub(super) enum Identity {
    Text(String),
    Number(u64),
}

impl Identity {
    fn matches(&self, expected: &str) -> bool {
        match self {
            Self::Text(value) => value == expected,
            Self::Number(value) => value.to_string() == expected,
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum AccessClaim {
    Encoded(String),
    Invalid(IgnoredAny),
}

#[derive(Deserialize)]
pub(super) struct Claims {
    iss: String,
    exp: u64,
    nbf: u64,
    repository_id: Identity,
    run_id: Identity,
    runner_id: Identity,
    job_id: Option<Identity>,
    ac: Option<AccessClaim>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Access {
    scope: String,
    permission: u32,
}

impl Claims {
    pub(super) fn scope(
        self,
        facts: &JobFacts,
        now: u64,
    ) -> Result<JobScope, UnknownJobScopeReason> {
        if self.iss != ISSUER {
            return Err(UnknownJobScopeReason::InvalidIssuer);
        }
        if self.nbf > self.exp {
            return Err(UnknownJobScopeReason::InvalidValidity);
        }
        if self.nbf > now.saturating_add(LEEWAY) {
            return Err(UnknownJobScopeReason::NotYetValid);
        }
        if now > self.exp.saturating_add(LEEWAY) {
            return Err(UnknownJobScopeReason::Expired);
        }
        if !self.repository_id.matches(&facts.repository.id.0) {
            return Err(UnknownJobScopeReason::RepositoryMismatch);
        }
        if !self.run_id.matches(&facts.assignment.run_id) {
            return Err(UnknownJobScopeReason::RunMismatch);
        }
        if !self.runner_id.matches(&facts.assignment.runner_id) {
            return Err(UnknownJobScopeReason::RunnerMismatch);
        }
        if let Some(job) = &facts.assignment.job_id
            && !self.job_id.is_some_and(|value| value.matches(&job.0))
        {
            return Err(UnknownJobScopeReason::JobMismatch);
        }
        let Some(AccessClaim::Encoded(encoded)) = self.ac else {
            return Err(UnknownJobScopeReason::InvalidAccessControl);
        };
        let Ok(access) = serde_json::from_str::<Vec<Access>>(&encoded) else {
            return Err(UnknownJobScopeReason::InvalidAccessControl);
        };
        if access.iter().any(|entry| entry.scope.is_empty()) {
            return Err(UnknownJobScopeReason::InvalidAccessControl);
        }
        let mut writers = access.iter().filter(|entry| entry.permission & 2 != 0);
        let Some(writer) = writers.next() else {
            return Err(UnknownJobScopeReason::NoWriteScope);
        };
        if writers.next().is_some() {
            return Err(UnknownJobScopeReason::MultipleWriteScopes);
        }
        Ok(JobScope::Checked {
            git_ref: GitRef(writer.scope.clone()),
            read_refs: access
                .into_iter()
                .filter(|entry| entry.permission & 1 != 0)
                .map(|entry| GitRef(entry.scope))
                .collect(),
        })
    }
}
