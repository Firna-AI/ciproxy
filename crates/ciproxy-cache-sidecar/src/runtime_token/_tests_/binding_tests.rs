//! Signed job claims accept only the exact registered runner's paired UUIDs.

use ciproxy_protocol::identifiers::GitRef;
use ciproxy_protocol::jobs::{JobScope, UnknownJobScopeReason};
use serde_json::json;

use crate::runtime_token::RuntimeTokens;

use super::support::{claims, facts, signed, verifier};

#[tokio::test]
async fn signed_job_claim_matches_either_bound_uuid_and_rejects_another_binding() {
    let checked = JobScope::Checked {
        git_ref: GitRef("refs/heads/main".into()),
        read_refs: vec![GitRef("refs/heads/main".into())],
    };
    let mismatch = JobScope::Unknown {
        reason: UnknownJobScopeReason::JobMismatch,
    };
    for (case, claim, expected) in [
        (
            "Worker UUID",
            Some("11111111-1111-4111-8111-111111111111"),
            checked.clone(),
        ),
        (
            "scale-set UUID",
            Some("22222222-2222-4222-8222-222222222222"),
            checked,
        ),
        (
            "another runner's binding UUID",
            Some("33333333-3333-4333-8333-333333333333"),
            mismatch.clone(),
        ),
        ("missing job claim", None, mismatch),
    ] {
        let mut payload = claims(json!([{"Scope":"refs/heads/main","Permission":3}]));
        payload.as_object_mut().unwrap().remove("job_id");
        if let Some(job_id) = claim {
            payload["job_id"] = json!(job_id);
        }
        assert_eq!(
            verifier().check(&signed(&payload), &facts()).await,
            expected,
            "{case}"
        );
    }
}
