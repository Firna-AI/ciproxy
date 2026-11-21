//! Superseded and renewed intents must not start another priced move.

use axum::http::StatusCode;
use ciproxy_store_interface::billing::payments::Settlement;
use reqwest::Method;

use super::credit_test_support::{pay, priced_requests, review, set_time, upgrade, work};
use super::fake_subscriptions::{SubscriptionOutcome, SubscriptionRequest};
use super::pending_credit_tests::machine_credit;
use super::support::{at, json};

#[tokio::test]
async fn cancel_and_app_removal_restore_prepared_credit_without_replaying_the_move() {
    for (removed, submitted, scheduled) in [
        (false, false, false),
        (true, false, false),
        (true, true, true),
    ] {
        let (dashboard, payer) = machine_credit().await;
        dashboard
            .fake
            .state
            .lock()
            .unwrap()
            .subscription
            .cancel_at_period_end = scheduled;
        let quote = review(&dashboard, &payer).await;
        if submitted {
            dashboard
                .fake
                .expect_subscription(upgrade(1795089600, -1000), SubscriptionOutcome::Reject(503));
        } else {
            dashboard.fake.state.lock().unwrap().lose_next_delete = true;
        }
        assert_eq!(
            pay(&dashboard, &payer, &quote).await.status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        let prepared = work(&dashboard, &quote);
        assert_eq!(prepared.submitted_at.is_some(), submitted);
        assert_eq!(
            prepared.retiring.as_deref(),
            if submitted {
                None
            } else {
                Some("ii_machine_credit")
            }
        );
        let mut canceled = dashboard.fake.state.lock().unwrap().subscription.clone();
        canceled.cancel_at_period_end = true;
        if !scheduled {
            dashboard.fake.expect_subscription(
                SubscriptionRequest::CancelAtPeriodEnd,
                SubscriptionOutcome::respond(canceled),
            );
        }
        if removed {
            dashboard
                .fixture
                .database
                .store
                .record_app_removal(
                    dashboard.fixture.workspace,
                    dashboard.fixture.organization.id,
                    at("2026-11-19T12:00:00Z"),
                )
                .unwrap();
            dashboard
                .fake
                .state
                .lock()
                .unwrap()
                .credits
                .as_mut()
                .unwrap()
                .fail_subscription_reads = 1;
            dashboard.pages.cancellations.reconcile().await.unwrap();
            let held = dashboard
                .fixture
                .database
                .store
                .billing(dashboard.fixture.workspace)
                .unwrap();
            assert_eq!(
                held.command.map(|command| command.id),
                if submitted { Some(prepared.id) } else { None }
            );
            assert_eq!(priced_requests(&dashboard), usize::from(submitted));
            dashboard.pages.cancellations.reconcile().await.unwrap();
        } else {
            let response = json(
                dashboard
                    .request(Some(&payer), Method::POST, "/billing/move-to-free", None)
                    .await,
                StatusCode::OK,
            )
            .await;
            assert_eq!(response["ok"], true);
        }
        let settled = work(&dashboard, &quote);
        assert_eq!(
            settled.settlement,
            if submitted {
                Settlement::Unresolved
            } else {
                Settlement::Restored
            },
            "removed={removed},submitted={submitted},scheduled={scheduled}"
        );
        assert_eq!(settled.pending, submitted);
        let unfinished = dashboard
            .fixture
            .database
            .store
            .try_lock_billing(dashboard.fixture.workspace)
            .unwrap()
            .unwrap()
            .payments()
            .unwrap();
        assert_eq!(unfinished.len(), usize::from(submitted));
        let state = dashboard
            .fixture
            .database
            .store
            .billing(dashboard.fixture.workspace)
            .unwrap();
        assert_eq!(state.command, None);
        assert_eq!(state.state.cancel_at, Some(at("2026-12-04T12:00:00Z")));
        assert_eq!(priced_requests(&dashboard), usize::from(submitted));
        let balance = {
            let remote = dashboard.fake.state.lock().unwrap();
            let ledger = remote.credits.as_ref().unwrap();
            (ledger.balance_cents, ledger.balance_transactions.len())
        };
        assert_eq!(balance, if submitted { (0, 0) } else { (-1000, 1) });
        dashboard.pages.cancellations.reconcile().await.unwrap();
        let repeated = dashboard
            .fake
            .state
            .lock()
            .unwrap()
            .credits
            .as_ref()
            .unwrap()
            .balance_transactions
            .len();
        assert_eq!(repeated, usize::from(!submitted));
        dashboard.close().await;
    }
}

#[tokio::test]
async fn renewal_abandons_unsubmitted_credit_and_holds_a_submitted_unknown_move() {
    for submitted in [false, true] {
        let (dashboard, payer) = machine_credit().await;
        set_time(&dashboard, "2026-12-04T11:59:00Z");
        let quote = review(&dashboard, &payer).await;
        if submitted {
            dashboard
                .fake
                .expect_subscription(upgrade(1796385540, -1000), SubscriptionOutcome::Reject(503));
        } else {
            dashboard.fake.state.lock().unwrap().lose_next_delete = true;
        }
        assert_eq!(
            pay(&dashboard, &payer, &quote).await.status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        let before = work(&dashboard, &quote);
        assert_eq!(before.submitted_at.is_some(), submitted);
        set_time(&dashboard, "2026-12-04T12:01:00Z");
        {
            let mut remote = dashboard.fake.state.lock().unwrap();
            for item in &mut remote.subscription.items.data {
                item.current_period_start = 1796385600;
                item.current_period_end = 1799064000;
            }
            let clock = remote.proration.as_mut().unwrap();
            clock.starts_at = 1796385600;
            clock.ends_at = 1799064000;
        }
        let response = json(pay(&dashboard, &payer, &quote).await, StatusCode::CONFLICT).await;
        assert_eq!(
            response["code"],
            if submitted {
                "conflict"
            } else {
                "review_required"
            }
        );
        let after = work(&dashboard, &quote);
        assert_eq!(after.quote, before.quote);
        assert_eq!(
            after.settlement,
            if submitted {
                Settlement::Unresolved
            } else {
                Settlement::Restored
            }
        );
        assert_eq!(priced_requests(&dashboard), usize::from(submitted));
        let balance = {
            let remote = dashboard.fake.state.lock().unwrap();
            let ledger = remote.credits.as_ref().unwrap();
            (ledger.balance_cents, ledger.balance_transactions.len())
        };
        assert_eq!(balance, if submitted { (0, 0) } else { (-1000, 1) });
        dashboard.close().await;
    }
}
