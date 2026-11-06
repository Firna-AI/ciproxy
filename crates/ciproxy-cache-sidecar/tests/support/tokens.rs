//! Private RSA fixture tokens and a local JWKS endpoint, never GitHub.

use std::ops::{Deref, DerefMut};

use axum::{Json, Router, routing};
use ciproxy_protocol::identifiers::{GitRef, JobEvent, OrganizationId, RepositoryId};
use ciproxy_protocol::jobs::{JobAssignment, JobFacts, Repository};
use ciproxy_protocol::secret::Secret;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

const PRIVATE_KEY: &[u8] = include_bytes!("../../src/runtime_token/_tests_/fixtures/key.pem");
const JWKS: &str = include_str!("../../src/runtime_token/_tests_/fixtures/jwks.json");

#[derive(Clone)]
pub struct JobSpec {
    pub facts: JobFacts,
    pub reference: String,
    pub base_ref: Option<GitRef>,
    pub token: Option<Secret>,
}

impl Deref for JobSpec {
    type Target = JobFacts;
    fn deref(&self) -> &Self::Target {
        &self.facts
    }
}
impl DerefMut for JobSpec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.facts
    }
}

impl JobSpec {
    pub fn runtime_token(&self) -> Secret {
        if let Some(token) = &self.token {
            return token.clone();
        }
        let mut access = vec![json!({"Scope":self.reference,"Permission":3})];
        if let Some(base) = &self.base_ref {
            access.push(json!({"Scope":base.0,"Permission":1}));
        }
        if self.reference != self.default_ref.0 {
            access.push(json!({"Scope":self.default_ref.0,"Permission":1}));
        }
        signed(
            &json!({"iss":"https://token.actions.githubusercontent.com", "nbf":999_000,"exp":2_000_000,
            "repository_id":self.repository.id.0,"run_id":self.assignment.run_id,"runner_id":self.assignment.runner_id,
            "job_id":self.assignment.job_id.as_ref().map(|id| &id.0),"ac":json!(access).to_string()}),
        )
    }
}

pub fn signed(claims: &Value) -> Secret {
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some("fixture-rsa".into());
    Secret::new(
        encode(
            &header,
            claims,
            &EncodingKey::from_rsa_pem(PRIVATE_KEY).unwrap(),
        )
        .unwrap(),
    )
}

pub fn facts(reference: &str) -> JobSpec {
    JobSpec {
        facts: JobFacts {
            organization_id: OrganizationId("1".into()),
            repository: Repository {
                id: RepositoryId("10".into()),
                full_name: "org/repo".into(),
            },
            assignment: JobAssignment {
                run_id: "31".into(),
                job_id: None,
                runner_id: "71".into(),
            },
            event: JobEvent("push".into()),
            head_repository: None,
            default_ref: GitRef("refs/heads/main".into()),
            sticky_docker: false,
        },
        reference: reference.into(),
        base_ref: None,
        token: None,
    }
}

pub(super) async fn serve_keys() -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/jwks", listener.local_addr().unwrap());
    let app = Router::new().route(
        "/jwks",
        routing::get(|| async { Json(serde_json::from_str::<Value>(JWKS).unwrap()) }),
    );
    let task = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (url, task)
}
