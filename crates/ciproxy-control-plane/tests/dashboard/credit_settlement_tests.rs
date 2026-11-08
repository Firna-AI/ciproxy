//! Frozen retries, definitive rejection, and unpaid credit restoration.

use axum::http::StatusCode;
use ciproxy_store_interface::billing::payments::{Settlement, ValidationFailure};
use ciproxy_store_interface::billing::state::Tier;
use serde_json::json;
use uuid::Uuid;

use super::credit_test_support::{
    business, pay, priced_requests, review, set_time, upgrade, void_invoice, work,
};
use super::fake_subscriptions::SubscriptionOutcome;
use super::pending_credit_tests::machine_credit;

#[tokio::test]
async fn definitively_uncharged_moves_restore_the_credit_once() {
    for (validation, amount, expected_today, receipts) in [
        (false, -1000_i64, "11500", 1_usize),
        (true, -1000, "11500", 1),
        (false, 1000, "13500", 2),
    ] {
        let (mut dashboard, payer) = machine_credit().await;
        if amount > 0 {
            let mut remote = dashboard.fake.state.lock().unwrap();
            let customer = remote.subscription.customer.clone();
            let ledger = remote.credits.as_mut().unwrap();
            ledger.pending.push(
                json!({"id":"ii_earlier_charge","amount":2000,"currency":"usd",
                "customer":customer,"invoice":null,"parent":null}),
            );
            ledger.page_size = 1;
            ledger
                .balance_transactions
                .push(json!({"id":"cbtxn_unrelated","customer":customer,
                "amount":0,"currency":"usd","metadata":{"ciproxy_payment_id":Uuid::new_v4()}}));
        }
        let quote = review(&dashboard, &payer).await;
        let outcome = if validation {
            SubscriptionOutcome::ProviderError {
                status: 400,
                kind: "invalid_request_error",
                code: "parameter_missing",
            }
        } else {
            dashboard
                .fake
                .state
                .lock()
                .unwrap()
                .credits
                .as_mut()
                .unwrap()
                .pending_payment = true;
            SubscriptionOutcome::respond(business(&dashboard))
        };
        dashboard
            .fake
            .expect_subscription(upgrade(1795089600, amount), outcome);
        let result = pay(&dashboard, &payer, &quote).await;
        assert_eq!(
            result.status(),
            if validation {
                StatusCode::SERVICE_UNAVAILABLE
            } else {
                StatusCode::OK
            }
        );
        if !validation {
            void_invoice(&dashboard);
            dashboard
                .fake
                .state
                .lock()
                .unwrap()
                .credits
                .as_mut()
                .unwrap()
                .lose_restore_response = true;
            dashboard.restart().await;
            dashboard.pages.cancellations.reconcile().await.unwrap();
        }
        let saved = work(&dashboard, &quote);
        assert_eq!(
            saved.settlement,
            Settlement::Restored,
            "validation={validation}"
        );
        assert!(!saved.pending);
        assert_eq!(
            saved.rejection.map(|proof| proof.reason),
            if validation {
                Some(ValidationFailure::MissingParameter)
            } else {
                None
            }
        );
        assert_eq!(
            dashboard
                .fixture
                .database
                .store
                .billing(dashboard.fixture.workspace)
                .unwrap()
                .command,
            None
        );
        let available = {
            let remote = dashboard.fake.state.lock().unwrap();
            let ledger = remote.credits.as_ref().unwrap();
            (
                ledger.balance_cents,
                ledger.pending.len(),
                ledger.balance_transactions.len(),
            )
        };
        assert_eq!(available, (amount, 0, receipts));
        let later = review(&dashboard, &payer).await;
        assert_eq!(later["today_cents"], expected_today);
        dashboard.pages.cancellations.reconcile().await.unwrap();
        let repeats = dashboard
            .fake
            .state
            .lock()
            .unwrap()
            .credits
            .as_ref()
            .unwrap()
            .balance_transactions
            .len();
        assert_eq!(repeats, receipts);
        assert_eq!(priced_requests(&dashboard), 1);
        let state = dashboard
            .fixture
            .database
            .store
            .billing(dashboard.fixture.workspace)
            .unwrap()
            .state;
        assert_eq!(state.tier, Tier::Team);
        dashboard.close().await;
    }
}

#[tokio::test]
async fn idempotency_rejection_keeps_uncertain_credit_without_a_second_priced_request() {
    let (dashboard, payer) = machine_credit().await;
    let quote = review(&dashboard, &payer).await;
    dashboard.fake.expect_subscription(
        upgrade(1795089600, -1000),
        SubscriptionOutcome::ProviderError {
            status: 400,
            kind: "idempotency_error",
            code: "parameter_missing",
        },
    );
    assert_eq!(
        pay(&dashboard, &payer, &quote).await.status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_ne!(
        dashboard
            .fixture
            .database
            .store
            .billing(dashboard.fixture.workspace)
            .unwrap()
            .command,
        None
    );
    set_time(&dashboard, "2026-11-20T12:01:00Z");
    dashboard.pages.cancellations.reconcile().await.unwrap();
    let saved = work(&dashboard, &quote);
    assert_eq!(saved.settlement, Settlement::Unresolved);
    assert_eq!(saved.rejection, None);
    let balance = {
        let remote = dashboard.fake.state.lock().unwrap();
        let ledger = remote.credits.as_ref().unwrap();
        (ledger.balance_cents, ledger.balance_transactions.len())
    };
    assert_eq!(balance, (0, 0));
    assert_eq!(priced_requests(&dashboard), 1);
    assert!(saved.pending);
    dashboard.close().await;
}
