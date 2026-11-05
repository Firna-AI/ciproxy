//! Signed scope shapes, identity binding, validity, and hostile token changes.

use ciproxy_protocol::identifiers::GitRef;
use ciproxy_protocol::jobs::{JobScope, UnknownJobScopeReason};
use ciproxy_protocol::secret::Secret;
use jsonwebtoken::Algorithm;
use serde_json::json;

use crate::runtime_token::RuntimeTokens;

use super::support::{NOW, claims, facts, signed, signed_header, verifier};

#[tokio::test]
async fn signed_scopes_preserve_every_observed_job_shape() {
    for (case, reference, reads) in [
        ("main push", "refs/heads/main", vec![]),
        ("main dispatch", "refs/heads/main", vec![]),
        ("branch push", "refs/heads/feature", vec!["refs/heads/main"]),
        (
            "branch dispatch",
            "refs/heads/feature",
            vec!["refs/heads/main"],
        ),
        (
            "tag named main",
            "refs/heads/refs/tags/main",
            vec!["refs/heads/main"],
        ),
        (
            "tag matching branch",
            "refs/heads/refs/tags/feature",
            vec!["refs/heads/main"],
        ),
        (
            "tag dispatch",
            "refs/heads/refs/tags/release",
            vec!["refs/heads/main"],
        ),
        (
            "PR merge",
            "refs/pull/7/merge",
            vec!["refs/heads/release", "refs/heads/main"],
        ),
        (
            "local reusable",
            "refs/heads/feature",
            vec!["refs/heads/main"],
        ),
        (
            "reusable at other ref",
            "refs/pull/7/merge",
            vec!["refs/heads/release", "refs/heads/main"],
        ),
    ] {
        let mut access = vec![json!({"Scope":reference,"Permission":3})];
        access.extend(
            reads
                .iter()
                .map(|scope| json!({"Scope":scope,"Permission":1})),
        );
        let token = signed(&claims(json!(access)));
        let mut expected_reads = vec![GitRef(reference.into())];
        expected_reads.extend(reads.into_iter().map(|scope| GitRef(scope.into())));
        assert_eq!(
            verifier().check(&token, &facts()).await,
            JobScope::Checked {
                git_ref: GitRef(reference.into()),
                read_refs: expected_reads
            },
            "{case}"
        );
    }
    for (case, access) in [
        (
            "PR target",
            json!([{"Scope":"refs/heads/main","Permission":1}]),
        ),
        (
            "cache-mode read",
            json!([{"Scope":"refs/heads/feature","Permission":1},{"Scope":"refs/heads/main","Permission":1}]),
        ),
    ] {
        assert_eq!(
            verifier().check(&signed(&claims(access)), &facts()).await,
            JobScope::Unknown {
                reason: UnknownJobScopeReason::NoWriteScope
            },
            "{case}"
        );
    }
}

#[tokio::test]
async fn rejects_wrong_assignment_invalid_times_and_malformed_access() {
    for (field, value, reason) in [
        (
            "iss",
            json!("https://another.example"),
            UnknownJobScopeReason::InvalidIssuer,
        ),
        (
            "repository_id",
            json!("11"),
            UnknownJobScopeReason::RepositoryMismatch,
        ),
        ("run_id", json!("32"), UnknownJobScopeReason::RunMismatch),
        (
            "runner_id",
            json!("72"),
            UnknownJobScopeReason::RunnerMismatch,
        ),
        (
            "job_id",
            json!("another-uuid"),
            UnknownJobScopeReason::JobMismatch,
        ),
        (
            "exp",
            json!(NOW - 31),
            UnknownJobScopeReason::InvalidValidity,
        ),
        ("nbf", json!(NOW + 31), UnknownJobScopeReason::NotYetValid),
        (
            "ac",
            json!("not JSON"),
            UnknownJobScopeReason::InvalidAccessControl,
        ),
        ("ac", json!([]), UnknownJobScopeReason::InvalidAccessControl),
        (
            "ac",
            json!(null),
            UnknownJobScopeReason::InvalidAccessControl,
        ),
        ("ac", json!("[]"), UnknownJobScopeReason::NoWriteScope),
        (
            "ac",
            json!(
                "[{\"Scope\":\"refs/heads/a\",\"Permission\":3},{\"Scope\":\"refs/heads/b\",\"Permission\":2}]"
            ),
            UnknownJobScopeReason::MultipleWriteScopes,
        ),
    ] {
        let mut payload = claims(json!([{"Scope":"refs/heads/main","Permission":3}]));
        payload[field] = value;
        assert_eq!(
            verifier().check(&signed(&payload), &facts()).await,
            JobScope::Unknown { reason },
            "{field}"
        );
    }
    let mut payload = claims(json!([{"Scope":"refs/heads/main","Permission":3}]));
    payload["nbf"] = json!(NOW - 100);
    payload["exp"] = json!(NOW - 31);
    assert_eq!(
        verifier().check(&signed(&payload), &facts()).await,
        JobScope::Unknown {
            reason: UnknownJobScopeReason::Expired
        }
    );
    payload.as_object_mut().unwrap().remove("runner_id");
    assert_eq!(
        verifier().check(&signed(&payload), &facts()).await,
        JobScope::Unknown {
            reason: UnknownJobScopeReason::MalformedToken
        }
    );
}

#[tokio::test]
async fn forged_access_and_missing_signature_never_grant_a_ref() {
    let verifier = verifier();
    let original = signed(&claims(
        json!([{"Scope":"refs/heads/feature","Permission":3}]),
    ));
    let changed = signed(&claims(json!([{"Scope":"refs/heads/main","Permission":3}])));
    let original_parts: Vec<_> = original.expose_secret().split('.').collect();
    let changed_parts: Vec<_> = changed.expose_secret().split('.').collect();
    for token in [
        Secret::new(format!(
            "{}.{}.{}",
            original_parts[0], changed_parts[1], original_parts[2]
        )),
        Secret::new(format!("{}.{}.", original_parts[0], original_parts[1])),
    ] {
        assert_eq!(
            verifier.check(&token, &facts()).await,
            JobScope::Unknown {
                reason: UnknownJobScopeReason::InvalidSignature
            }
        );
    }
    let payload = claims(json!([{"Scope":"refs/heads/main","Permission":3}]));
    for (token, reason) in [
        (Secret::new(""), UnknownJobScopeReason::MissingToken),
        (
            Secret::new("not-a-jwt"),
            UnknownJobScopeReason::MalformedToken,
        ),
        (
            signed_header(&payload, Algorithm::HS256, Some("fixture-rsa")),
            UnknownJobScopeReason::UnsupportedAlgorithm,
        ),
        (
            signed_header(&payload, Algorithm::RS256, None),
            UnknownJobScopeReason::MissingKeyId,
        ),
        (
            signed_header(&payload, Algorithm::RS256, Some("unknown")),
            UnknownJobScopeReason::UnknownKey,
        ),
    ] {
        assert_eq!(
            verifier.check(&token, &facts()).await,
            JobScope::Unknown { reason }
        );
    }
}

#[tokio::test]
async fn optional_job_identity_and_short_clock_leeway_keep_valid_assignments() {
    let mut expected = facts();
    expected.assignment.job_id = None;
    let mut payload = claims(json!([{"Scope":"refs/heads/main","Permission":3}]));
    payload["job_id"] = json!("different-optional-job");
    payload["repository_id"] = json!(10);
    payload["nbf"] = json!(NOW + 30);
    payload["exp"] = json!(NOW + 30);
    let result = JobScope::Checked {
        git_ref: GitRef("refs/heads/main".into()),
        read_refs: vec![GitRef("refs/heads/main".into())],
    };
    assert_eq!(verifier().check(&signed(&payload), &expected).await, result);
    payload["nbf"] = json!(NOW - 100);
    payload["exp"] = json!(NOW - 30);
    assert_eq!(verifier().check(&signed(&payload), &expected).await, result);
}
