//! Boat routes exercise real persistence, key checks, and browser authorization.

use std::sync::atomic::Ordering;

use axum::http::StatusCode;
use ciproxy_store_interface::dashboard::records::Role;
use serde_json::json;

use super::fixture::Fixture;
use super::fixture_io::json;
use super::overflow_support::{service, start};

#[tokio::test]
async fn boat_connection_requires_admin_positive_limit_and_accepted_key() {
    let mut fixture = Fixture::new();
    let ports = service(&mut fixture);
    let admin = fixture.login(7, Some(Role::Admin)).await;
    let body = json!({"api_key":"fake-boat-key","spend_limit_cents":"20000"});
    let configured = fixture.state.overflow.take();
    assert_eq!(
        fixture
            .request(
                Some(&admin),
                "POST",
                &fixture.path("/overflow/connection"),
                Some(body.clone())
            )
            .await
            .status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
    fixture.state.overflow = configured;
    assert_eq!(
        fixture
            .request(None, "GET", &fixture.path("/overflow"), None)
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
    let reader = fixture.login(8, Some(Role::Viewer)).await;
    assert_eq!(
        fixture
            .request(
                Some(&reader),
                "GET",
                &fixture.path("/overflow/summary"),
                None
            )
            .await
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        fixture
            .request(
                Some(&reader),
                "POST",
                &fixture.path("/overflow/connection"),
                Some(body.clone())
            )
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    let mut invalid = body.clone();
    invalid["spend_limit_cents"] = json!("0");
    assert_eq!(
        fixture
            .request(
                Some(&admin),
                "POST",
                &fixture.path("/overflow/connection"),
                Some(invalid)
            )
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
    ports.reject.store(true, Ordering::SeqCst);
    let rejected = fixture
        .request(
            Some(&admin),
            "POST",
            &fixture.path("/overflow/connection"),
            Some(body.clone()),
        )
        .await;
    assert_eq!(rejected.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(json(rejected).await["code"], "boat_key_rejected");
    assert_eq!(
        fixture
            .database
            .store
            .boat_accounts(fixture.workspace)
            .unwrap()
            .len(),
        0
    );
    ports.reject.store(false, Ordering::SeqCst);
    assert_eq!(
        fixture
            .request(
                Some(&admin),
                "POST",
                &fixture.path("/overflow/connection"),
                Some(body)
            )
            .await
            .status(),
        StatusCode::OK
    );
    let saved = json(
        fixture
            .request(Some(&admin), "GET", &fixture.path("/overflow"), None)
            .await,
    )
    .await;
    assert_eq!(saved["account"]["name"], "Customer Boat");
    assert_eq!(saved["spend_limit_cents"], "20000");
    assert_eq!(saved["next_month_at"], "2026-12-01T00:00:00Z");
    assert!(!saved.to_string().contains("fake-boat-key"));
    let request = json!({"enabled":false,"wait_seconds":120,"spend_limit_cents":"5000"});
    assert_eq!(
        fixture
            .request(
                Some(&admin),
                "PUT",
                &fixture.path("/overflow"),
                Some(request)
            )
            .await
            .status(),
        StatusCode::OK
    );
    let stored = fixture.database.store.settings(fixture.workspace).unwrap();
    assert!(!stored.overflow_enabled);
    assert_eq!(stored.boat_spend_limit_usd_micros, Some(50_000_000));
}

#[tokio::test]
async fn replacement_and_disconnect_preserve_a_running_sandbox_and_saved_reconnect_limit() {
    let mut fixture = Fixture::new();
    let ports = service(&mut fixture);
    let admin = fixture.login(7, Some(Role::Admin)).await;
    let connection = json!({"api_key":"fake-boat-key","spend_limit_cents":"20000"});
    assert_eq!(
        fixture
            .request(
                Some(&admin),
                "POST",
                &fixture.path("/overflow/connection"),
                Some(connection.clone())
            )
            .await
            .status(),
        StatusCode::OK
    );
    let old = fixture
        .database
        .store
        .boat_accounts(fixture.workspace)
        .unwrap()[0]
        .id;
    let old_key = fixture
        .database
        .store
        .encrypted_boat_key(fixture.workspace, old)
        .unwrap()
        .unwrap();
    start(&fixture).await;
    assert_eq!(
        fixture
            .request(
                Some(&admin),
                "POST",
                &fixture.path("/overflow/connection"),
                Some(connection)
            )
            .await
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        fixture
            .database
            .store
            .encrypted_boat_key(fixture.workspace, old)
            .unwrap(),
        Some(old_key.clone())
    );
    assert_eq!(
        fixture
            .request(
                Some(&admin),
                "DELETE",
                &fixture.path("/overflow/connection"),
                None
            )
            .await
            .status(),
        StatusCode::OK
    );
    let view = json(
        fixture
            .request(Some(&admin), "GET", &fixture.path("/overflow"), None)
            .await,
    )
    .await;
    assert_eq!(view["account"], serde_json::Value::Null);
    assert_eq!(view["spend_limit_cents"], "20000");
    assert_eq!(ports.deletes.load(Ordering::SeqCst), 0);
    assert_eq!(
        fixture
            .database
            .store
            .encrypted_boat_key(fixture.workspace, old)
            .unwrap(),
        Some(old_key.clone())
    );
}
