//! Provider routes name the provider and save only its policy.

use axum::http::StatusCode;
use ciproxy_store_interface::dashboard::records::Role;
use serde_json::json;

use super::fixture::Fixture;
use super::fixture_io::json;
use super::overflow_support::service;

#[tokio::test]
async fn provider_limit_write_is_independent_of_workspace_policy() {
    let fixture = Fixture::new();
    let admin = fixture.login(7, Some(Role::Admin)).await;
    let policy = fixture.database.store.settings(fixture.workspace).unwrap();
    let limit = fixture.path("/overflow/providers/boat/spend-limit");
    let saved = fixture
        .request(
            Some(&admin),
            "PUT",
            &limit,
            Some(json!({"spend_limit_cents":"12345"})),
        )
        .await;
    assert_eq!(saved.status(), StatusCode::OK, "save the provider limit");
    assert_eq!(json(saved).await, json!({"ok":true, "provider":"boat"}));
    assert_eq!(
        fixture.database.store.settings(fixture.workspace).unwrap(),
        policy
    );
    let settings = json(
        fixture
            .request(Some(&admin), "GET", &fixture.path("/overflow"), None)
            .await,
    )
    .await;
    let provider = &settings["providers"][0];
    assert_eq!(provider["provider"], "boat");
    assert_eq!(provider["available"], false);
    assert_eq!(provider["has_billing_account_choice"], true);
    assert_eq!(provider["offered"], false);
    assert_eq!(provider["spend_limit_cents"], "12345");
    let unknown = fixture
        .request(
            Some(&admin),
            "PUT",
            &fixture.path("/overflow/providers/unknown/spend-limit"),
            Some(json!({"spend_limit_cents":"67890"})),
        )
        .await;
    assert_eq!(unknown.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn unavailable_provider_keeps_account_and_limit_and_can_disconnect() {
    let mut fixture = Fixture::new();
    service(&mut fixture);
    let admin = fixture.login(7, Some(Role::Admin)).await;
    let connection = fixture.path("/overflow/providers/boat/connection");
    let response = fixture
        .request(
            Some(&admin),
            "POST",
            &connection,
            Some(json!({"api_key":"fixture-key", "spend_limit_cents":"20000"})),
        )
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(json(response).await, json!({"provider":"boat", "ok":true}));

    fixture.state.overflow = None;
    let overflow = fixture.path("/overflow");
    let snapshot = json(fixture.request(Some(&admin), "GET", &overflow, None).await).await;
    let provider = &snapshot["providers"][0];
    assert_eq!(provider["available"], false);
    assert_eq!(provider["account"]["name"], "Customer Boat");
    assert_eq!(provider["spend_limit_cents"], "20000");
    assert_eq!(provider["spend"]["confirmed_cents"], "0");

    let disconnected = fixture
        .request(Some(&admin), "DELETE", &connection, None)
        .await;
    assert_eq!(disconnected.status(), StatusCode::OK);
    assert_eq!(
        json(disconnected).await,
        json!({"provider":"boat", "ok":true})
    );
    let saved = json(fixture.request(Some(&admin), "GET", &overflow, None).await).await;
    assert_eq!(saved["providers"][0]["account"], serde_json::Value::Null);
    assert_eq!(saved["providers"][0]["spend_limit_cents"], "20000");
    assert_eq!(saved["providers"][0]["offered"], true);
}
