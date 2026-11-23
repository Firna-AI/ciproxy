//! Pending setup crosses the terms page and starts GitHub proof only on Continue.

use std::collections::BTreeSet;

use axum::body::Body;
use axum::http::{Response, StatusCode, header};
use chrono::Duration;
use serde_json::Value;

use super::adapter_server::Server;
use super::fixture::Fixture;
use super::fixture_io::{CallbackOutcome, cookie, json, oauth_state};
use super::setup_support::{
    begin, browser, configure, finish, finish_callback, pending_cookie, start_authorization,
};

async fn setup_fixture() -> (Fixture, Server) {
    let server = Server::new().await;
    let mut fixture = Fixture::new();
    configure(&mut fixture, &server);
    (fixture, server)
}

async fn pending_setup(fixture: &Fixture) -> String {
    pending_cookie(&begin(fixture, None, "?installation_id=101").await)
}

async fn signed_in_session(fixture: &Fixture, authorization: Response<Body>) -> Value {
    let signed_in = browser(&finish(fixture, authorization, CallbackOutcome::Accepted).await);
    let response = fixture
        .request(Some(&signed_in), "GET", "/session", None)
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    json(response).await
}

#[tokio::test]
async fn setup_return_opens_terms_with_only_a_pending_cookie() {
    let (fixture, _server) = setup_fixture().await;
    let returned = begin(&fixture, None, "?installation_id=101").await;
    let pending = pending_cookie(&returned);
    let cookies = returned.headers().get_all(header::SET_COOKIE);
    assert_eq!(cookies.iter().count(), 1);
    let attributes = cookies
        .iter()
        .next()
        .unwrap()
        .to_str()
        .unwrap()
        .split(';')
        .skip(1)
        .map(str::trim)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        attributes,
        BTreeSet::from([
            "Path=/",
            "Secure",
            "SameSite=Lax",
            "Max-Age=600",
            "HttpOnly"
        ])
    );

    let authorization = start_authorization(&fixture, None, None, "").await;
    finish_callback(
        &fixture,
        &oauth_state(&authorization),
        &pending,
        CallbackOutcome::Refused,
    )
    .await;
}

#[tokio::test]
async fn resume_proves_installation_with_the_browser_state_at_continue() {
    let (fixture, _server) = setup_fixture().await;
    let prior = fixture.login(7, None).await;
    let pending = pending_setup(&fixture).await;
    fixture.advance(Duration::minutes(10) - Duration::seconds(1));

    let authorization =
        start_authorization(&fixture, Some(&prior), Some(&pending), "?resume=setup").await;
    let session = signed_in_session(&fixture, authorization).await;
    assert_eq!(session["setup"]["kind"], "organization");
    assert_eq!(session["setup"]["installation_id"], "101");
    assert_eq!(session["setup"]["github_id"], "100");
    assert_eq!(session["expires_at"], "2026-12-06T12:00:00Z");
}

#[tokio::test]
async fn expired_pending_setup_resumes_as_plain_sign_in() {
    let (fixture, _server) = setup_fixture().await;
    let pending = pending_setup(&fixture).await;
    fixture.advance(Duration::minutes(10) + Duration::seconds(1));

    let authorization = start_authorization(&fixture, None, Some(&pending), "?resume=setup").await;
    let session = signed_in_session(&fixture, authorization).await;
    assert_eq!(session.get("setup"), Some(&Value::Null));
}

#[tokio::test]
async fn changed_pending_setup_resumes_as_plain_sign_in() {
    let (fixture, _server) = setup_fixture().await;
    let mut pending = pending_setup(&fixture).await;
    let start = pending.rfind('.').unwrap() + 1;
    let replacement = if pending.as_bytes()[start] == b'A' {
        "B"
    } else {
        "A"
    };
    pending.replace_range(start..start + 1, replacement);

    let authorization = start_authorization(&fixture, None, Some(&pending), "?resume=setup").await;
    let session = signed_in_session(&fixture, authorization).await;
    assert_eq!(session.get("setup"), Some(&Value::Null));
}

#[tokio::test]
async fn plain_sign_in_replaces_pending_setup_and_later_resume_stays_plain() {
    let (fixture, _server) = setup_fixture().await;
    let pending = pending_setup(&fixture).await;

    let plain = start_authorization(&fixture, None, Some(&pending), "").await;
    assert_eq!(plain.status(), StatusCode::SEE_OTHER);
    let replacement = cookie(&plain, "__Host-ciproxy-oauth");
    let session = signed_in_session(&fixture, plain).await;
    assert_eq!(session.get("setup"), Some(&Value::Null), "plain sign-in");

    let resumed = start_authorization(&fixture, None, Some(&replacement), "?resume=setup").await;
    let session = signed_in_session(&fixture, resumed).await;
    assert_eq!(session.get("setup"), Some(&Value::Null), "OAuth cookie");

    let resumed = start_authorization(&fixture, None, None, "?resume=setup").await;
    let session = signed_in_session(&fixture, resumed).await;
    assert_eq!(session.get("setup"), Some(&Value::Null), "no cookie");
}

#[tokio::test]
async fn other_resume_values_do_not_take_over_pending_setup() {
    let (fixture, _server) = setup_fixture().await;
    let pending = pending_setup(&fixture).await;
    for query in ["?resume=other", "?resume=setup&resume=other"] {
        let authorization = start_authorization(&fixture, None, Some(&pending), query).await;
        let session = signed_in_session(&fixture, authorization).await;
        assert_eq!(session.get("setup"), Some(&Value::Null), "{query}");
    }
}
