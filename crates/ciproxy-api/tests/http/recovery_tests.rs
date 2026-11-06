//! HTTP sign-in, role isolation, retention, and owner recovery requirements.

use std::sync::atomic::Ordering;

use axum::http::{StatusCode, header};
use chrono::Duration;
use ciproxy_store_interface::dashboard::records::{Membership, Role};
use serde_json::json;

use super::fixture::Fixture;
use super::fixture_io::json;

#[tokio::test]
async fn owner_claim_rechecks_idle_admin_before_recovery_and_requires_ownership() {
    let fixture = Fixture::new();
    let _admin = fixture.login(7, Some(Role::Admin)).await;
    fixture.identity.memberships.lock().unwrap().insert(
        8,
        vec![Membership {
            github_id: 100,
            owner: true,
        }],
    );
    let owner = fixture.login(8, Some(Role::Viewer)).await;
    let mut second = fixture.organization.clone();
    second.id = uuid::Uuid::new_v4();
    second.github_id = 200;
    second.installation_id = 201;
    second.login = "other".to_owned();
    fixture
        .database
        .store
        .install_organization(&second)
        .unwrap();
    fixture.remote.organizations.lock().unwrap().push(second);
    assert_eq!(
        fixture
            .request(Some(&owner), "POST", &fixture.path("/claim-admin"), None)
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    fixture.advance(Duration::hours(1));
    fixture.identity.memberships.lock().unwrap().insert(
        8,
        vec![
            Membership {
                github_id: 100,
                owner: true,
            },
            Membership {
                github_id: 200,
                owner: true,
            },
        ],
    );
    let all_owner = fixture.login(8, Some(Role::Viewer)).await;
    fixture.identity.fail.store(true, Ordering::SeqCst);
    assert_eq!(
        fixture
            .request(
                Some(&all_owner),
                "POST",
                &fixture.path("/claim-admin"),
                None
            )
            .await
            .status(),
        StatusCode::OK
    );
    fixture.identity.fail.store(false, Ordering::SeqCst);
    fixture.identity.memberships.lock().unwrap().insert(
        8,
        vec![Membership {
            github_id: 100,
            owner: true,
        }],
    );
    let owner = fixture.login(8, Some(Role::Viewer)).await;
    fixture
        .identity
        .memberships
        .lock()
        .unwrap()
        .insert(7, Vec::new());
    fixture.advance(Duration::hours(1));
    assert_eq!(
        fixture
            .request(Some(&owner), "POST", &fixture.path("/claim-admin"), None)
            .await
            .status(),
        StatusCode::OK
    );
    let outsider = fixture.login(9, Some(Role::Viewer)).await;
    assert_eq!(
        fixture
            .request(Some(&outsider), "POST", &fixture.path("/claim-admin"), None)
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn build_header_and_json_errors_cover_missing_routes_and_extractor_errors() {
    let fixture = Fixture::new();
    let browser = fixture.login(7, Some(Role::Admin)).await;
    for (method, path, body, status) in [
        ("GET", "/missing".to_owned(), None, StatusCode::NOT_FOUND),
        (
            "POST",
            "/session".to_owned(),
            None,
            StatusCode::METHOD_NOT_ALLOWED,
        ),
        (
            "GET",
            "/workspaces/invalid/settings".to_owned(),
            None,
            StatusCode::BAD_REQUEST,
        ),
        (
            "PUT",
            fixture.path("/settings"),
            Some(json!({"bad":true})),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
    ] {
        let response = fixture.request(Some(&browser), method, &path, body).await;
        assert_eq!(response.status(), status);
        assert!(json(response).await["code"].is_string());
    }
    fixture.identity.fail.store(true, Ordering::SeqCst);
    fixture.advance(Duration::hours(1));
    let response = fixture
        .request(Some(&browser), "GET", "/session", None)
        .await;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert!(!response.headers().contains_key(header::SET_COOKIE));
}
