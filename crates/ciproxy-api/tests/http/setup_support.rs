//! Real setup HTTP adapters and browser redirects for the setup scenarios.

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, Response, StatusCode, header};
use ciproxy_api::access::AccessService;
use ciproxy_api::crypto::AeadCredentialCipher;
use ciproxy_api::identity::http::{GithubOAuthClient, OAuthConfig};
use ciproxy_api::installations::GithubInstallationAdmin;
use ciproxy_api::router::router;
use ciproxy_github::auth::AppJwt;
use reqwest::{Client, redirect::Policy};
use secrecy::SecretString;
use tower::ServiceExt;

use super::adapter_server::Server;
use super::fixture::{Browser, Fixture};
use super::fixture_io::{cookie, oauth_state};

struct Signer(SecretString);
impl AppJwt for Signer {
    fn token(&self) -> ciproxy_github::error::Result<SecretString> {
        Ok(self.0.clone())
    }
}

pub(super) fn configure(fixture: &mut Fixture, server: &Server) {
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
        fixture.runtime.clone(),
    ));
    fixture.state.access = Arc::new(AccessService::new(
        fixture.database.store.clone(),
        identity,
        Arc::new(AeadCredentialCipher::new(&[17; 32])),
        fixture.runtime.clone(),
    ));
    fixture.state.installations = Arc::new(GithubInstallationAdmin::new(
        http,
        server.origin.parse().unwrap(),
        Arc::new(Signer(server.token.clone())),
    ));
}

pub(super) async fn finish(fixture: &Fixture, begin: Response<Body>) -> Response<Body> {
    assert_eq!(begin.status(), StatusCode::SEE_OTHER);
    let nonce = oauth_state(&begin);
    let binding = cookie(&begin, "__Host-ciproxy-oauth");
    let request = Request::builder()
        .uri(format!(
            "/api/auth/github/callback?code=synthetic&state={nonce}"
        ))
        .header(header::COOKIE, format!("__Host-ciproxy-oauth={binding}"))
        .body(Body::empty())
        .unwrap();
    let response = Router::new()
        .nest("/api", router(fixture.state.clone()))
        .oneshot(request)
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get("x-ciproxy-build").unwrap(),
        "http-test-build"
    );
    response
}

pub(super) fn browser(response: &Response<Body>) -> Browser {
    assert_eq!(
        response
            .headers()
            .get(header::LOCATION)
            .and_then(|value| value.to_str().ok()),
        Some("/")
    );
    Browser {
        cookie: cookie(response, "__Host-ciproxy"),
        csrf: cookie(response, "__Host-ciproxy-csrf"),
    }
}

pub(super) async fn begin(
    fixture: &Fixture,
    prior: Option<&Browser>,
    query: &str,
) -> Response<Body> {
    let mut request = Request::builder().uri(format!("/api/github/setup{query}"));
    if let Some(prior) = prior {
        request = request.header(header::COOKIE, format!("__Host-ciproxy={}", prior.cookie));
    }
    Router::new()
        .nest("/api", router(fixture.state.clone()))
        .oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap()
}

pub(super) async fn old_callback_status(fixture: &Fixture) -> StatusCode {
    Router::new()
        .nest("/api", router(fixture.state.clone()))
        .oneshot(
            Request::builder()
                .uri("/api/auth/callback")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
        .status()
}
