//! The published fee remains counted after zero-runtime completion and expiry.

use chrono::Duration;
use ciproxy_overflow::api::{Overflow, StartOutcome};
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::overflow::record::StartPhase;
use rust_decimal::Decimal;
use serde_json::{Value, json};

use crate::tls::facts::{identity, limits};
use crate::tls::peer::step;
use crate::tls::server::Server;

use super::authority_support::{DisableAt, Provider};
use super::fixtures::{at, id, request};
use super::harness::{Harness, Runtime};

#[tokio::test]
async fn zero_quote_saves_the_highest_fee_through_zero_runtime_completion_and_expiry() {
    let mut prices = limits(true);
    prices["nextStartDollars"] = json!(0);
    let server = Server::new(vec![
        step("GET", "/api/v1/me", 200, identity()),
        step("GET", "/api/v1/limits", 200, prices.clone()),
        step("GET", "/api/v1/me", 200, identity()),
        step("GET", "/api/v1/limits", 200, prices),
        step(
            "GET",
            "/api/v1/sandboxes/bx_23456789/usage?",
            200,
            usage(1, true),
        ),
        step(
            "GET",
            "/api/v1/sandboxes/bx_23456789/usage?",
            200,
            usage(1, true),
        ),
        step(
            "GET",
            "/api/v1/sandboxes/bx_23456789/usage?",
            200,
            usage(2, false),
        ),
        step(
            "GET",
            "/api/v1/sandboxes/bx_23456789/usage?",
            200,
            usage(2, false),
        ),
    ])
    .await;
    let provider = Provider::new(DisableAt::Never);
    let mut backend = provider.backend;
    backend.account = server.backend.account.clone();
    let harness = Harness::new(backend, (), Runtime::Start);
    let started = harness.service.start(id(1), request()).await.unwrap();
    assert!(matches!(started, StartOutcome::Started(_)));
    assert_eq!(
        harness.start(id(11)).start_fee_bound_usd,
        Some(Decimal::new(7, 2))
    );
    assert_eq!(harness.start(id(11)).start_usd, None);

    *harness.now.lock().unwrap() = at() + Duration::seconds(1);
    let pending = harness.service.view(id(1)).await.unwrap();
    assert_eq!(pending.spend.confirmed_usd, Decimal::ZERO);
    assert_eq!(pending.spend.pending_usd, Decimal::new(502, 3));
    assert_eq!(pending.scaler_account(at()).unwrap().spend.spent, 502_000);

    *harness.now.lock().unwrap() = at() + Duration::seconds(2);
    harness
        .service
        .report_end(Secret::new("callback-token-for-one-sandbox"), id(12))
        .await
        .unwrap();
    let completed = harness.service.view(id(1)).await.unwrap();
    assert_eq!(completed.starts[0].phase, StartPhase::Ended);
    assert_eq!(completed.starts[0].runtime[0].usd, Decimal::ZERO);
    assert!(completed.starts[0].runtime_complete);
    assert_eq!(completed.starts[0].start_usd, None);
    assert_eq!(completed.spend.confirmed_usd, Decimal::ZERO);
    assert_eq!(completed.spend.pending_usd, Decimal::new(7, 2));
    assert_eq!(completed.scaler_account(at()).unwrap().spend.spent, 70_000);

    let after_expiry = at() + Duration::hours(7);
    *harness.now.lock().unwrap() = after_expiry;
    harness.set_checked(after_expiry);
    let expired = harness.service.view(id(1)).await.unwrap();
    assert_eq!(
        expired.starts[0].start_fee_bound_usd,
        Some(Decimal::new(7, 2))
    );
    assert_eq!(expired.spend.confirmed_usd, Decimal::ZERO);
    assert_eq!(expired.spend.pending_usd, Decimal::ZERO);
    assert_eq!(expired.spend.settled_bound_usd, Decimal::new(7, 2));
    assert_eq!(expired.scaler_account(at()).unwrap().spend.spent, 70_000);
    server.assert_consumed();
    assert_eq!(server.captured()[1].body, Value::Null);
    assert_eq!(
        server
            .captured()
            .iter()
            .filter(|request| request.path == "/api/v1/limits")
            .count(),
        2
    );
    assert_eq!(provider.counts.lock().unwrap().creates, 1);
    assert_eq!(provider.counts.lock().unwrap().deletes, 1);
}

fn usage(seconds: i64, running: bool) -> Value {
    json!({"ok":true,"type":"sandbox.usage","sandboxId":"bx_23456789",
        "since":at(),"until":at() + Duration::seconds(seconds),"dollars":0,"running":running})
}
