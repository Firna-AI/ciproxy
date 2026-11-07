//! Display rounds aggregated costs while saved provider bounds stay exact.

use std::sync::atomic::Ordering;

use axum::http::StatusCode;
use chrono::Duration;
use ciproxy_store_interface::dashboard::records::Role;
use rust_decimal::Decimal;
use serde_json::json;

use super::fixture::Fixture;
use super::fixture_io::json;
use super::overflow_support::{service, start};

#[tokio::test]
async fn spend_rounds_after_aggregation_and_keeps_settled_bounds_unconfirmed() {
    let mut fixture = Fixture::new();
    let ports = service(&mut fixture);
    let admin = fixture.login(7, Some(Role::Admin)).await;
    assert_eq!(
        fixture
            .request(
                Some(&admin),
                "POST",
                &fixture.path("/overflow/connection"),
                Some(json!({"api_key":"fake-key","spend_limit_cents":"20000"}))
            )
            .await
            .status(),
        StatusCode::OK
    );
    start(&fixture).await;
    start(&fixture).await;
    *ports.now.lock().unwrap() += Duration::seconds(1);
    for (cost, expected) in [
        (Decimal::new(25, 4), "1"),
        (Decimal::new(2, 3), "0"),
        (Decimal::new(9975, 4), "200"),
    ] {
        *ports.usage_usd.lock().unwrap() = Some(cost);
        let view = json(
            fixture
                .request(Some(&admin), "GET", &fixture.path("/overflow"), None)
                .await,
        )
        .await;
        assert_eq!(view["spend"]["confirmed_cents"], expected);
        assert_eq!(view["spend"]["unconfirmed_cents"], "14");
        assert_eq!(view["spend"]["settled_bound_cents"], "0");
        assert_eq!(view["spend"]["bounded"], true);
    }
    *ports.usage_usd.lock().unwrap() = None;
    *ports.now.lock().unwrap() += Duration::hours(7);
    let view = json(
        fixture
            .request(Some(&admin), "GET", &fixture.path("/overflow"), None)
            .await,
    )
    .await;
    assert_eq!(view["spend"]["confirmed_cents"], "200");
    assert_eq!(view["spend"]["unconfirmed_cents"], "14");
    assert_eq!(view["spend"]["settled_bound_cents"], "14");
}

#[tokio::test]
async fn refreshed_key_rejection_date_appears_on_the_same_http_response() {
    let mut fixture = Fixture::new();
    let ports = service(&mut fixture);
    let admin = fixture.login(7, Some(Role::Admin)).await;
    assert_eq!(
        fixture
            .request(
                Some(&admin),
                "POST",
                &fixture.path("/overflow/connection"),
                Some(json!({"api_key":"fake-key","spend_limit_cents":"20000"}))
            )
            .await
            .status(),
        StatusCode::OK
    );
    *ports.now.lock().unwrap() += Duration::minutes(5);
    ports.reject.store(true, Ordering::SeqCst);
    let view = json(
        fixture
            .request(Some(&admin), "GET", &fixture.path("/overflow"), None)
            .await,
    )
    .await;
    assert_eq!(view["account"]["rejected_at"], "2026-11-06T12:05:00Z");
    assert_eq!(view["spend_limit_cents"], "20000");
}
