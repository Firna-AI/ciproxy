//! The production root serves every page and raw signed delivery on one host.

use axum::http::StatusCode;
use ciproxy_api::runtime::Runtime;
use ciproxy_billing::stripe::wire::types::{CheckoutStatus, PaymentStatus};
use ciproxy_store_interface::billing::state::Tier;
use ciproxy_store_interface::dashboard::records::Role;
use reqwest::Method;
use serde_json::{Value, json};

use super::support::{Dashboard, json, signature};

#[tokio::test]
async fn composed_pages_and_stripe_hook_share_the_bound_dashboard_host() {
    let dashboard = Dashboard::new().await;
    let billing = dashboard.fixture.login(7, Some(Role::Billing)).await;
    let viewer = dashboard.fixture.login(8, Some(Role::Viewer)).await;
    let job = dashboard.queued_job();
    for (suffix, banner) in [
        ("/jobs/live", "/banner"),
        ("/jobs/history", "/banner"),
        ("/savings", "/banner"),
        ("/billing", "/banner"),
        ("/plans", "/usage/banner"),
        ("/banner", "/banner"),
    ] {
        let body = json(
            dashboard
                .request(Some(&viewer), Method::GET, suffix, None)
                .await,
            StatusCode::OK,
        )
        .await;
        assert_eq!(
            body.pointer(banner),
            Some(&Value::Null),
            "page {suffix} supplies its shared banner"
        );
    }
    let live = json(
        dashboard
            .request(Some(&viewer), Method::GET, "/jobs/live", None)
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(live["waiting_count"], "1");
    let detail = json(
        dashboard
            .request(Some(&viewer), Method::GET, &format!("/jobs/{job}"), None)
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(detail["job"]["name"], "composed-build");
    let review = json(
        dashboard
            .request(
                Some(&billing),
                Method::GET,
                "/checkout/review?tier=team&start_blocks=2&machines=1",
                None,
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(review["today_cents"], "8900");
    assert_eq!(review["prorated"], false);
    let selection = json!({"tier":"team","add_ons":{"start_blocks":2,"machines":1}});
    assert_eq!(
        dashboard
            .request(None, Method::GET, "/billing", None)
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        dashboard
            .request(
                Some(&viewer),
                Method::POST,
                "/checkout",
                Some(selection.clone())
            )
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    let no_csrf = dashboard
        .client
        .post(format!(
            "{}/api{}",
            dashboard.origin,
            dashboard.fixture.path("/checkout")
        ))
        .header("Cookie", format!("__Host-ciproxy={}", billing.cookie))
        .header("Origin", &dashboard.fixture.state.config.public_origin)
        .json(&selection)
        .send()
        .await
        .unwrap();
    assert_eq!(no_csrf.status(), StatusCode::FORBIDDEN);
    let open = json(
        dashboard
            .request(Some(&billing), Method::POST, "/checkout", Some(selection))
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(open["state"], "open");
    {
        let mut remote = dashboard.fake.state.lock().unwrap();
        let checkout = remote.checkout.as_mut().unwrap();
        checkout.status = CheckoutStatus::Complete;
        checkout.payment_status = PaymentStatus::Paid;
        checkout.customer = Some("cus_recorded".into());
        checkout.subscription = Some("sub_recorded".into());
        remote.subscription.metadata = checkout.metadata.clone();
    }
    let now = Runtime::now(&*dashboard.fixture.runtime);
    let event = json!({"id":"evt_composed_checkout","created":now.timestamp(),"type":"checkout.session.completed","data":{"object":{"id":"cs_recorded"}}}).to_string();
    for expected in ["applied", "duplicate"] {
        let response = dashboard
            .client
            .post(format!("{}/hooks/stripe", dashboard.origin))
            .header("Stripe-Signature", signature(event.as_bytes(), now))
            .body(event.clone())
            .send()
            .await
            .unwrap();
        assert_eq!(json(response, StatusCode::OK).await, expected);
    }
    let returned = json(
        dashboard
            .request(
                Some(&billing),
                Method::POST,
                "/checkout/return",
                Some(json!({"session_id":"cs_recorded"})),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(returned["outcome"]["state"], "confirmed");
    assert_eq!(
        dashboard
            .fixture
            .database
            .store
            .billing(dashboard.fixture.workspace)
            .unwrap()
            .state
            .tier,
        Tier::Team
    );
    let ignored = br#"{ "id":"evt_composed_ignored", "created":1793966400, "type":"charge.succeeded", "data":{"object":{"id":"ch_composed","object":"charge","amount":8900}} }"#;
    let response = dashboard
        .client
        .post(format!("{}/hooks/stripe", dashboard.origin))
        .header("Stripe-Signature", signature(ignored, now))
        .body(ignored.as_slice())
        .send()
        .await
        .unwrap();
    assert_eq!(json(response, StatusCode::OK).await, "ignored");
    let invalid = dashboard
        .client
        .post(format!("{}/hooks/stripe", dashboard.origin))
        .header("Stripe-Signature", "t=1,v1=bad")
        .body(ignored.as_slice())
        .send()
        .await
        .unwrap();
    assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);
    let alias = dashboard
        .client
        .post(format!("{}/api/hooks/stripe", dashboard.origin))
        .send()
        .await
        .unwrap();
    assert_eq!(alias.status(), StatusCode::NOT_FOUND);
    let body: Value = alias.json().await.unwrap();
    assert_eq!(body["code"], "not_found");
    dashboard.fake.probe().check();
    dashboard.close().await;
}
