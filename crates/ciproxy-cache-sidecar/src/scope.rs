//! Repository isolation and GitHub's branch-first lookup order.

use ciproxy_protocol::identifiers::JobId;
use ciproxy_protocol::jobs::JobFacts;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct Scope {
    pub organization: String,
    pub repository: String,
    pub reference: String,
}

pub(crate) fn write_scope(facts: &JobFacts, job: &JobId) -> Scope {
    let reference = if can_save(facts, true) {
        facts.git_ref.0.clone()
    } else {
        format!("private-job/{}", job.0)
    };
    Scope {
        organization: facts.organization_id.0.clone(),
        repository: facts.repository.id.0.clone(),
        reference,
    }
}

pub(crate) fn read_scopes(facts: &JobFacts, job: &JobId) -> Vec<Scope> {
    let own = write_scope(facts, job);
    let mut scopes = vec![own.clone()];
    if !facts.event.0.starts_with("pull_request") {
        scopes.push(Scope {
            reference: facts.git_ref.0.clone(),
            ..own.clone()
        });
    }
    if facts.event.0.starts_with("pull_request")
        && let Some(base) = &facts.base_ref
    {
        scopes.push(Scope {
            reference: base.0.clone(),
            ..own.clone()
        });
    }
    scopes.push(Scope {
        reference: facts.default_ref.0.clone(),
        ..own
    });
    scopes.dedup();
    scopes
}

pub(crate) fn can_save(facts: &JobFacts, permissions: bool) -> bool {
    permissions
        && (trusted_event(facts)
            || facts.event.0.starts_with("pull_request") && private_pr_ref(facts))
        && (facts.git_ref != facts.default_ref
            || facts
                .head_repository
                .as_ref()
                .is_none_or(|head| head.id == facts.repository.id))
}

fn trusted_event(facts: &JobFacts) -> bool {
    matches!(
        facts.event.0.as_str(),
        "push"
            | "workflow_dispatch"
            | "repository_dispatch"
            | "delete"
            | "registry_package"
            | "page_build"
            | "schedule"
    )
}

fn private_pr_ref(facts: &JobFacts) -> bool {
    facts.event.0 != "pull_request_target"
        && facts
            .git_ref
            .0
            .strip_prefix("refs/pull/")
            .and_then(|value| value.strip_suffix("/merge"))
            .is_some_and(|number| number.parse::<u64>().is_ok())
}
