//! A rotated GitHub pair survives a later membership outage.

use std::sync::atomic::Ordering;

use axum::http::StatusCode;
use chrono::Duration;
use ciproxy_store_interface::dashboard::records::Role;

use super::adapter_server::Server;
use super::fixture::Fixture;
use super::setup_support::configure;

#[tokio::test]
async fn rotated_credentials_survive_membership_failure_and_retry_without_signin() {
    let server = Server::new().await;
    let mut fixture = Fixture::new();
    configure(&mut fixture, &server);
    let browser = fixture.login(7, Some(Role::Admin)).await;
    let before = fixture
        .database
        .store
        .dashboard_user_by_github(7)
        .unwrap()
        .unwrap();
    let memberships = fixture
        .database
        .store
        .dashboard_memberships(before.id)
        .unwrap();
    fixture.advance(Duration::hours(8));
    server.oauth.fail_memberships.store(true, Ordering::SeqCst);
    let failed = fixture
        .request(Some(&browser), "GET", &fixture.path("/machines"), None)
        .await;
    assert_eq!(failed.status(), StatusCode::SERVICE_UNAVAILABLE);
    let rotated = fixture
        .database
        .store
        .dashboard_user(before.id)
        .unwrap()
        .unwrap();
    assert!(
        rotated.credentials != before.credentials,
        "rotation must persist before membership verification"
    );
    assert_eq!(rotated.checked_at, before.checked_at);
    let unchanged = fixture
        .database
        .store
        .dashboard_memberships(before.id)
        .unwrap();
    assert_eq!(
        unchanged
            .iter()
            .map(|member| (member.github_id, member.owner))
            .collect::<Vec<_>>(),
        memberships
            .iter()
            .map(|member| (member.github_id, member.owner))
            .collect::<Vec<_>>()
    );
    let retry = fixture
        .request(Some(&browser), "GET", &fixture.path("/machines"), None)
        .await;
    assert_eq!(retry.status(), StatusCode::OK);
    assert_eq!(server.oauth.refreshes.load(Ordering::SeqCst), 1);
    assert_eq!(
        fixture
            .database
            .store
            .dashboard_user(before.id)
            .unwrap()
            .unwrap()
            .checked_at,
        "2026-11-06T20:00:00Z"
            .parse::<chrono::DateTime<chrono::Utc>>()
            .unwrap()
    );
}
