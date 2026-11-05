//! Repository isolation using checked scopes and private unknown sessions.

use ciproxy_protocol::jobs::{JobFacts, JobScope};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct Scope {
    pub organization: String,
    pub repository: String,
    pub reference: String,
}

pub(crate) fn write_scope(facts: &JobFacts, scope: &JobScope, private_ref: &str) -> Scope {
    let reference = match scope {
        JobScope::Checked { git_ref, .. } if can_save(facts, scope, true) => git_ref.0.clone(),
        _ => private_ref.to_owned(),
    };
    Scope {
        organization: facts.organization_id.0.clone(),
        repository: facts.repository.id.0.clone(),
        reference,
    }
}

pub(crate) fn read_scopes(facts: &JobFacts, scope: &JobScope, private_ref: &str) -> Vec<Scope> {
    let own = write_scope(facts, scope, private_ref);
    let mut scopes = Vec::new();
    if let JobScope::Checked { git_ref, read_refs } = scope {
        if own.reference != git_ref.0 {
            scopes.push(own.clone());
        }
        for reference in read_refs
            .iter()
            .filter(|reference| *reference == git_ref)
            .chain(read_refs)
        {
            if reference != &facts.default_ref
                && !scopes.iter().any(|scope| scope.reference == reference.0)
            {
                scopes.push(Scope {
                    reference: reference.0.clone(),
                    ..own.clone()
                });
            }
        }
    }
    scopes.push(Scope {
        reference: facts.default_ref.0.clone(),
        ..own
    });
    scopes
}

pub(crate) fn can_save(facts: &JobFacts, scope: &JobScope, permissions: bool) -> bool {
    if !permissions {
        return false;
    }
    let JobScope::Checked { git_ref, .. } = scope else {
        return true;
    };
    (trusted_event(facts)
        || facts.event.0.starts_with("pull_request") && private_pr_ref(facts, &git_ref.0))
        && (git_ref != &facts.default_ref
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

fn private_pr_ref(facts: &JobFacts, reference: &str) -> bool {
    facts.event.0 != "pull_request_target"
        && reference
            .strip_prefix("refs/pull/")
            .and_then(|value| value.strip_suffix("/merge"))
            .is_some_and(|number| number.parse::<u64>().is_ok())
}
