//! Restarted paid attempts retain the quote and consume each earlier source once.

use axum::http::StatusCode;
use ciproxy_billing::stripe::wire::types::Status;
use ciproxy_store_interface::billing::payments::{InvoiceAudit, Settlement};
use ciproxy_store_interface::billing::state::Tier;
use serde_json::json;

use super::credit_test_support::{business, pay, priced_requests, review, set_time, upgrade, work};
use super::fake_subscriptions::SubscriptionOutcome;
use super::pending_credit_tests::machine_credit;
use super::support::{at, json};

#[tokio::test]
async fn pending_credit_retry_keeps_the_quote_and_retires_its_source_once() {
    for applied_before_loss in [false, true] {
        let (mut dashboard, payer) = machine_credit().await;
        let quote = review(&dashboard, &payer).await;
        let outcome = if applied_before_loss {
            dashboard.fake.state.lock().unwrap().lose_next_post = true;
            SubscriptionOutcome::respond(business(&dashboard))
        } else {
            SubscriptionOutcome::Reject(503)
        };
        dashboard
            .fake
            .expect_subscription(upgrade(1795089600, -1000), outcome);
        assert_eq!(
            pay(&dashboard, &payer, &quote).await.status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        let before = work(&dashboard, &quote);
        assert_eq!(before.retired, vec!["ii_machine_credit"]);
        assert_ne!(before.submitted_at, None);
        dashboard.restart().await;
        if applied_before_loss {
            set_time(&dashboard, "2027-01-04T12:01:00Z");
            {
                let mut remote = dashboard.fake.state.lock().unwrap();
                let mut later = remote.credits.as_ref().unwrap().invoices[0].clone();
                later["id"] = json!("in_later_void_renewal");
                later["billing_reason"] = json!("subscription_cycle");
                later["status"] = json!("void");
                later["amount_paid"] = json!(0);
                later["amount_due"] = json!(29900);
                remote
                    .credits
                    .as_mut()
                    .unwrap()
                    .invoices
                    .insert(0, later.clone());
                remote.subscription.status = Status::Canceled;
                remote.subscription.ended_at = Some(1799064000);
                remote.subscription.latest_invoice = Some(serde_json::from_value(later).unwrap());
            }
            dashboard.pages.cancellations.reconcile().await.unwrap();
            assert_eq!(priced_requests(&dashboard), 1);
        } else {
            set_time(&dashboard, "2026-11-19T12:40:00Z");
            dashboard.fake.expect_subscription(
                upgrade(1795089600, -1000),
                SubscriptionOutcome::respond(business(&dashboard)),
            );
            let result = json(pay(&dashboard, &payer, &quote).await, StatusCode::OK).await;
            assert_eq!(result["state"], "confirmed");
            let requests = dashboard
                .fake
                .subscription_mutations()
                .into_iter()
                .filter(|request| {
                    request.value("payment_behavior") == Some("pending_if_incomplete")
                })
                .collect::<Vec<_>>();
            assert_eq!(requests.len(), 2);
            assert_eq!(requests[0].key, requests[1].key);
            assert_eq!(requests[0].form, requests[1].form);
            let state = dashboard
                .fixture
                .database
                .store
                .billing(dashboard.fixture.workspace)
                .unwrap()
                .state;
            assert_eq!(state.paid_period_start, Some(at("2026-11-04T12:00:00Z")));
            assert_eq!(state.paid_period_end, Some(at("2026-12-04T12:00:00Z")));
        }
        let after = work(&dashboard, &quote);
        assert_eq!(after.quote, before.quote);
        assert_eq!(after.settlement, Settlement::Consumed);
        assert_eq!(
            after.audit,
            InvoiceAudit::Matched {
                amount_cents: 11500
            }
        );
        assert_eq!(after.restoration, None);
        assert!(!after.pending);
        let facts = {
            let remote = dashboard.fake.state.lock().unwrap();
            let ledger = remote.credits.as_ref().unwrap();
            (
                ledger.charges.clone(),
                ledger.pending.len(),
                ledger.balance_cents,
                ledger.balance_transactions.len(),
                remote
                    .requests
                    .iter()
                    .filter(|request| request.method == "DELETE")
                    .count(),
            )
        };
        assert_eq!(facts, (vec![11500], 0, 0, 0, 1));
        let tier = dashboard
            .fixture
            .database
            .store
            .billing(dashboard.fixture.workspace)
            .unwrap()
            .state
            .tier;
        assert_eq!(
            tier,
            if applied_before_loss {
                Tier::Free
            } else {
                Tier::Business
            }
        );
        dashboard.close().await;
    }
}
