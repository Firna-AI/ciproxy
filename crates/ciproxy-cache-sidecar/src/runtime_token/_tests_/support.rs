//! Literal assignment and private test-key fixtures, with mocked IO and time.

use std::sync::Arc;

use ciproxy_protocol::identifiers::{GitRef, JobEvent, JobId, OrganizationId, RepositoryId};
use ciproxy_protocol::jobs::{JobAssignment, JobFacts, Repository};
use ciproxy_protocol::secret::Secret;
use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde_json::{Value, json};
use unimock::{MockFn, Unimock, matching};

use crate::clock::ClockMock;
use crate::runtime_token::TokenVerifier;
use crate::runtime_token::keys::CachedKeys;
use crate::runtime_token::source::KeySourceMock;

pub(super) const NOW: u64 = 1_000_000;

pub(super) fn keys() -> JwkSet {
    serde_json::from_str(include_str!("fixtures/jwks.json")).unwrap()
}

pub(super) fn verifier() -> TokenVerifier {
    let clock = Arc::new(Unimock::new(
        ClockMock::now.each_call(matching!()).returns(NOW),
    ));
    let source = Arc::new(Unimock::new(
        KeySourceMock::fetch
            .each_call(matching!())
            .answers(&|_| Ok(keys())),
    ));
    TokenVerifier::new(Arc::new(CachedKeys::new(source, clock.clone())), clock)
}

pub(super) fn facts() -> JobFacts {
    JobFacts {
        organization_id: OrganizationId("1".into()),
        repository: Repository {
            id: RepositoryId("10".into()),
            full_name: "org/repo".into(),
        },
        assignment: JobAssignment {
            run_id: "31".into(),
            job_id: Some(JobId("job-uuid".into())),
            runner_id: "71".into(),
        },
        event: JobEvent("push".into()),
        head_repository: None,
        default_ref: GitRef("refs/heads/main".into()),
        sticky_docker: false,
    }
}

pub(super) fn claims(access: Value) -> Value {
    json!({"iss":"https://token.actions.githubusercontent.com", "nbf": NOW - 10, "exp": NOW + 100,
        "repository_id":"10", "run_id":"31", "runner_id":"71", "job_id":"job-uuid", "ac":access.to_string()})
}

pub(super) fn signed(claims: &Value) -> Secret {
    signed_header(claims, Algorithm::RS256, Some("fixture-rsa"))
}

pub(super) fn signed_header(claims: &Value, algorithm: Algorithm, kid: Option<&str>) -> Secret {
    let mut header = Header::new(algorithm);
    header.kid = kid.map(str::to_owned);
    let key = if algorithm == Algorithm::RS256 {
        EncodingKey::from_rsa_pem(include_bytes!("fixtures/key.pem")).unwrap()
    } else {
        EncodingKey::from_secret(b"wrong-algorithm")
    };
    Secret::new(encode(&header, claims, &key).unwrap())
}
