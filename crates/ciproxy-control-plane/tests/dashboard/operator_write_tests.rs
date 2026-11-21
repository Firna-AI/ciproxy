//! Failed diagnostic writes keep settled money and confirmed paid access.

use axum::http::StatusCode;
use ciproxy_billing::alerts::{AlertReason, BillingAlert};
use ciproxy_store_interface::billing::payments::{InvoiceAudit, Settlement};
use ciproxy_store_interface::billing::state::{SubscriptionState, Tier};
use diesel::connection::SimpleConnection;
use diesel::{Connection, PgConnection};
use reqwest::Method;
use serde_json::Value;
use uuid::Uuid;

use super::credit_test_support::{business, pay, priced_requests, review, set_time, upgrade, work};
use super::fake_subscriptions::SubscriptionOutcome;
use super::pending_credit_tests::machine_credit;
use super::support::{Dashboard, json};

#[tokio::test]
async fn audit_write_failure_confirms_payment_allows_review_and_recovers_one_mismatch_alert() {
    let (mut dashboard, payer) = machine_credit().await;
    let quote = review(&dashboard, &payer).await;
    assert_eq!(quote["today_cents"], "11500");
    dashboard
        .fake
        .state
        .lock()
        .unwrap()
        .credits
        .as_mut()
        .unwrap()
        .force_charge_difference = 500;
    dashboard.fake.expect_subscription(
        upgrade(1795089600, -1000),
        SubscriptionOutcome::respond(business(&dashboard)),
    );
    reject_mismatch_writes(&dashboard.fixture.database.url);

    let response = json(pay(&dashboard, &payer, &quote).await, StatusCode::OK).await;
    assert_eq!(response["state"], "confirmed");
    let failed = work(&dashboard, &quote);
    let quote_id = quote["quote"]["id"].as_str().unwrap().parse().unwrap();
    let invoice_id = {
        let remote = dashboard.fake.state.lock().unwrap();
        remote.credits.as_ref().unwrap().invoices[0]["id"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    assert_eq!(failed.quote.today_cents, 11500);
    assert_eq!(failed.invoice_id.as_deref(), Some(invoice_id.as_str()));
    assert_eq!(failed.settlement, Settlement::Consumed);
    assert_eq!(failed.audit, InvoiceAudit::Waiting);
    assert_eq!(failed.restoration, None);
    assert!(failed.pending);
    assert_eq!(mismatch_alerts(&dashboard), Vec::new());
    assert_payment_preserved(&dashboard);

    set_time(&dashboard, "2026-11-19T12:31:00Z");
    let next = json(
        dashboard
            .request(
                Some(&payer),
                Method::GET,
                "/checkout/review?tier=team&start_blocks=0&machines=0",
                None,
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(next["choice"]["selection"]["tier"], "team");
    assert_eq!(next["today_cents"], "0");
    assert_ne!(next["quote"]["id"], quote["quote"]["id"]);
    let retained = dashboard
        .fixture
        .database
        .store
        .try_lock_billing(dashboard.fixture.workspace)
        .unwrap()
        .unwrap()
        .payment(failed.id)
        .unwrap()
        .unwrap();
    assert_eq!(retained, failed);
    let pruned = dashboard
        .fixture
        .database
        .store
        .try_lock_billing(dashboard.fixture.workspace)
        .unwrap()
        .unwrap()
        .quote(quote_id)
        .unwrap();
    assert_eq!(pruned, None);
    assert_payment_preserved(&dashboard);

    allow_mismatch_writes(&dashboard.fixture.database.url);
    dashboard.restart().await;
    reconcile(&dashboard).await;
    assert_recovered(&dashboard, &quote, quote_id, failed.id, &invoice_id);
    assert_payment_preserved(&dashboard);
    reconcile(&dashboard).await;
    assert_recovered(&dashboard, &quote, quote_id, failed.id, &invoice_id);
    assert_payment_preserved(&dashboard);
    dashboard.close().await;
}

fn reject_mismatch_writes(database_url: &str) {
    let mut connection = PgConnection::establish(database_url).unwrap();
    connection
        .batch_execute(
            r#"
            CREATE FUNCTION reject_mismatch_audit() RETURNS trigger
            LANGUAGE plpgsql AS $$
            BEGIN
                RAISE EXCEPTION 'synthetic mismatch audit write failure';
            END;
            $$;
            CREATE TRIGGER reject_mismatch_audit
            BEFORE UPDATE OF state_json ON billing_payments
            FOR EACH ROW
            WHEN ((NEW.state_json::jsonb -> 'audit') ? 'Mismatch')
            EXECUTE FUNCTION reject_mismatch_audit();
            "#,
        )
        .unwrap();
}

fn allow_mismatch_writes(database_url: &str) {
    let mut connection = PgConnection::establish(database_url).unwrap();
    connection
        .batch_execute(
            "DROP TRIGGER reject_mismatch_audit ON billing_payments;
             DROP FUNCTION reject_mismatch_audit();",
        )
        .unwrap();
}

fn assert_recovered(
    dashboard: &Dashboard,
    quote: &Value,
    quote_id: Uuid,
    command_id: Uuid,
    invoice_id: &str,
) {
    let recovered = dashboard
        .fixture
        .database
        .store
        .try_lock_billing(dashboard.fixture.workspace)
        .unwrap()
        .unwrap()
        .payment(command_id)
        .unwrap()
        .unwrap();
    assert_eq!(
        recovered.quote.payment.id.to_string(),
        quote["quote"]["id"].as_str().unwrap()
    );
    assert_eq!(recovered.id, command_id);
    assert_eq!(recovered.quote.payment.id, quote_id);
    assert_eq!(recovered.quote.today_cents, 11500);
    assert_eq!(recovered.invoice_id.as_deref(), Some(invoice_id));
    assert_eq!(recovered.settlement, Settlement::Consumed);
    assert_eq!(recovered.restoration, None);
    assert!(!recovered.pending);
    assert_eq!(
        recovered.audit,
        InvoiceAudit::Mismatch {
            amount_cents: 12000,
            alerted: true,
        }
    );
    assert_eq!(
        mismatch_alerts(dashboard),
        vec![BillingAlert {
            workspace_id: dashboard.fixture.workspace,
            quote_id,
            command_id,
            invoice_id: Some(invoice_id.into()),
            expected_cents: 11500,
            actual_cents: Some(12000),
            reason: AlertReason::InvoiceMismatch,
        }]
    );
}

fn mismatch_alerts(dashboard: &Dashboard) -> Vec<BillingAlert> {
    let events = dashboard.alerts.0.lock().unwrap().clone();
    events
        .into_iter()
        .filter(|event| event.reason == AlertReason::InvoiceMismatch)
        .collect()
}

fn assert_payment_preserved(dashboard: &Dashboard) {
    let record = dashboard
        .fixture
        .database
        .store
        .billing(dashboard.fixture.workspace)
        .unwrap();
    assert_eq!(record.state.tier, Tier::Business);
    assert_eq!(record.state.status, SubscriptionState::Active);
    assert_eq!(record.state.cancel_at, None);
    assert_eq!(record.command, None);
    let money = {
        let remote = dashboard.fake.state.lock().unwrap();
        let ledger = remote.credits.as_ref().unwrap();
        (
            ledger.charges.clone(),
            ledger.balance_cents,
            ledger.balance_transactions.len(),
            remote.subscription.cancel_at_period_end,
        )
    };
    assert_eq!(money, (vec![12000], 0, 0, false));
    assert_eq!(priced_requests(dashboard), 1);
    assert_eq!(dashboard.fake.subscription_mutations().len(), 2);
}

async fn reconcile(dashboard: &Dashboard) {
    let billing = dashboard.pages.services.billing.clone();
    let workspace = dashboard.fixture.workspace;
    tokio::task::spawn_blocking(move || billing.reconcile_payments(workspace))
        .await
        .unwrap()
        .unwrap();
}
