//! Real OAuth, App-JWT metadata, and SDK validation errors through dashboard HTTP.

use std::sync::Arc;

use async_trait::async_trait;
use axum::http::{StatusCode, header};
use chrono::{DateTime, Utc};
use ciproxy_api::access::AccessService;
use ciproxy_api::crypto::AeadCredentialCipher;
use ciproxy_api::identity::http::{GithubOAuthClient, OAuthConfig};
use ciproxy_api::installations::GithubInstallationAdmin;
use ciproxy_api::runtime::Runtime;
use ciproxy_github::auth::AppJwt;
use ciproxy_github::scale_sets::sdk::SdkScaleSetApi;
use ciproxy_github::scale_sets::{ScaleSetApi, ScaleSetFactory};
use ciproxy_store_interface::dashboard::records::Role;
use ciproxy_store_interface::organizations::Organization;
use reqwest::{Client, redirect::Policy};
use scaleset::{Credentials, RetryPolicy};
use secrecy::{ExposeSecret, SecretString};
use serde_json::json;
use url::Url;
use uuid::Uuid;

use super::adapter_server::Server;
use super::fixture::Fixture;
use super::fixture_io::{CallbackOutcome, json};
use super::ports::Inputs;
use super::setup_support::{browser, finish};

const VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";

struct PkceRuntime(Arc<Inputs>);
impl Runtime for PkceRuntime {
    fn now(&self) -> DateTime<Utc> {
        Runtime::now(&*self.0)
    }
    fn identifier(&self) -> Uuid {
        Uuid::new_v4()
    }
    fn secret(&self) -> ciproxy_api::error::Result<SecretString> {
        Ok(VERIFIER.into())
    }
}

struct Signer(SecretString);
impl AppJwt for Signer {
    fn token(&self) -> ciproxy_github::error::Result<SecretString> {
        Ok(self.0.clone())
    }
}
impl scaleset::JwtProvider for Signer {
    async fn token(&self) -> std::result::Result<String, scaleset::BoxError> {
        Ok(self.0.expose_secret().to_owned())
    }
}
struct Factory {
    origin: String,
    token: SecretString,
}
#[async_trait]
impl ScaleSetFactory for Factory {
    async fn for_organization(
        &self,
        organization: &Organization,
    ) -> ciproxy_github::error::Result<Arc<dyn ScaleSetApi>> {
        let credentials = Credentials::github_app_with_provider(
            organization.installation_id,
            Signer(self.token.clone()),
        )
        .unwrap();
        let http = Client::builder().redirect(Policy::none()).build().unwrap();
        let client = scaleset::Client::builder(
            format!("{}{}", self.origin, organization.login),
            credentials,
        )
        .http_client(http.clone())
        .allow_http()
        .retry_policy(RetryPolicy::none())
        .build()
        .unwrap();
        Ok(Arc::new(SdkScaleSetApi::new(client, http)))
    }
}

#[tokio::test]
async fn production_oauth_and_installation_metadata_verify_identity_and_all_pages() {
    let server = Server::new().await;
    let mut fixture = Fixture::new();
    let http = Client::builder().redirect(Policy::none()).build().unwrap();
    let identity = Arc::new(GithubOAuthClient::new(
        http.clone(),
        OAuthConfig {
            client_id: "app-client".to_owned(),
            client_secret: server.token.clone(),
            callback_url: "https://app.ciproxy.example/api/auth/github/callback".to_owned(),
            github: server.origin.parse().unwrap(),
            api: server.origin.parse().unwrap(),
        },
        Arc::new(PkceRuntime(fixture.runtime.clone())),
    ));
    fixture.state.access = Arc::new(AccessService::new(
        fixture.database.store.clone(),
        identity,
        Arc::new(AeadCredentialCipher::new(&[17; 32])),
        Arc::new(PkceRuntime(fixture.runtime.clone())),
    ));
    fixture.state.installations = Arc::new(GithubInstallationAdmin::new(
        http,
        server.origin.parse().unwrap(),
        Arc::new(Signer(server.token.clone())),
    ));
    fixture
        .database
        .store
        .ensure_dashboard_initial_admin(fixture.workspace, 7, fixture.runtime.now())
        .unwrap();
    let begin = fixture.request(None, "GET", "/auth/github", None).await;
    let authorization = Url::parse(
        begin
            .headers()
            .get(header::LOCATION)
            .unwrap()
            .to_str()
            .unwrap(),
    )
    .unwrap();
    let query = authorization
        .query_pairs()
        .collect::<std::collections::HashMap<_, _>>();
    let returned = finish(&fixture, begin, CallbackOutcome::Accepted).await;
    let browser = browser(&returned);
    assert_eq!(
        query
            .get("code_challenge_method")
            .map(|value| value.as_ref()),
        Some("S256")
    );
    assert_eq!(
        query.get("code_challenge").map(|value| value.as_ref()),
        Some("E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM")
    );
    {
        let verifiers = server.oauth.verifiers.lock().unwrap();
        assert_eq!(verifiers.len(), 1);
        assert_eq!(verifiers[0].expose_secret(), VERIFIER);
    }
    let session = json(
        fixture
            .request(Some(&browser), "GET", "/session", None)
            .await,
    )
    .await;
    assert!(session["setup"].is_null());
    assert_eq!(
        session["pending_install_requests"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        session["pending_install_requests"][0]["organization"],
        "waiting-org"
    );
    let memberships = fixture
        .database
        .store
        .dashboard_memberships(
            fixture
                .database
                .store
                .dashboard_user_by_github(7)
                .unwrap()
                .unwrap()
                .id,
        )
        .unwrap();
    assert_eq!(memberships.len(), 101);
    assert!(
        memberships
            .iter()
            .any(|membership| membership.github_id == 100 && membership.owner)
    );
    assert!(
        !memberships
            .iter()
            .any(|membership| membership.github_id == 999)
    );
    assert_eq!(
        fixture
            .database
            .store
            .organizations(fixture.workspace)
            .unwrap()
            .len(),
        1
    );
}

#[tokio::test]
async fn github_name_refusal_keeps_the_actual_sdk_validation_message() {
    let server = Server::new().await;
    let mut fixture = Fixture::new();
    let browser = fixture.login(7, Some(Role::Admin)).await;
    fixture.state.scale_sets = Arc::new(Factory {
        origin: server.origin.clone(),
        token: server.token.clone(),
    });
    let response=fixture.request(Some(&browser),"POST",&fixture.path("/labels"),Some(json!({"name":"refused label","vcpus":2,"memory_gb":8,"disk_gb":20,"allow_machines":true,"allow_boat":true,"when_nothing_fits":"wait"}))).await;
    if response.status() != StatusCode::UNPROCESSABLE_ENTITY {
        println!("upstream paths: {:?}", server.seen.lock().unwrap());
    }
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let problem = json(response).await;
    assert_eq!(problem["message"], "GitHub refused this exact label name.");
    assert_eq!(
        fixture
            .database
            .store
            .labels(fixture.workspace)
            .unwrap()
            .len(),
        1
    );
}
