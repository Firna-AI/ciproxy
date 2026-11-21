//! Paid invoice diagnostics persist without changing confirmed entitlement.

use axum::http::StatusCode;
use ciproxy_billing::alerts::{AlertReason, BillingAlert};
use ciproxy_billing::catalog::PriceCode;
use ciproxy_store_interface::billing::payments::{InvoiceAudit, PaymentWork, Settlement};
use ciproxy_store_interface::billing::state::{SubscriptionState, Tier};
use reqwest::Method;
use serde_json::{Value, json};
use uuid::Uuid;

use super::fake_subscriptions::{ChangeMode, ItemChange, SubscriptionOutcome, SubscriptionRequest};
use super::fixture::Browser;
use super::pending_credit_tests::machine_credit;
use super::support::{Dashboard, at, json};

#[tokio::test]
async fn paid_invoice_mismatch_alerts_once_without_rejecting_or_reversing_payment() {
    let (mut dashboard, payer, review) = reviewed_credit().await;
    dashboard
        .fake
        .state
        .lock()
        .unwrap()
        .credits
        .as_mut()
        .unwrap()
        .force_charge_difference = 500;
    pay(&dashboard, &payer, &review).await;
    let (command, invoice) = payment_identity(&dashboard);
    let quote = review["quote"]["id"].as_str().unwrap().parse().unwrap();
    assert_mismatch(&dashboard, quote, command, &invoice);
    assert_payment_preserved(&dashboard);

    dashboard.restart().await;
    reconcile(&dashboard).await;
    assert_mismatch(&dashboard, quote, command, &invoice);
    assert_payment_preserved(&dashboard);
    dashboard.close().await;
}

#[tokio::test]
async fn invoice_read_outage_confirms_payment_and_recovers_its_exact_invoice_after_restart() {
    let (mut dashboard, payer, review) = reviewed_credit().await;
    {
        let mut remote = dashboard.fake.state.lock().unwrap();
        let ledger = remote.credits.as_mut().unwrap();
        ledger.force_charge_difference = 500;
        ledger.invoice_reads_fail = true;
    }
    pay(&dashboard, &payer, &review).await;
    let (command, invoice) = payment_identity(&dashboard);
    let quote = review["quote"]["id"].as_str().unwrap().parse().unwrap();
    let waiting = payment_work(&dashboard, command);
    assert_eq!(waiting.audit, InvoiceAudit::Waiting);
    assert!(waiting.pending);
    assert_eq!(waiting.restoration, None);
    assert_eq!(mismatch_alerts(&dashboard), Vec::new());
    assert_payment_preserved(&dashboard);

    dashboard.restart().await;
    {
        let mut remote = dashboard.fake.state.lock().unwrap();
        let subscription = remote.subscription.id.clone();
        let ledger = remote.credits.as_mut().unwrap();
        let mut renewal = ledger.invoices[0].clone();
        renewal["id"] = json!("in_later_renewal");
        renewal["created"] = json!(1796385600);
        renewal["billing_reason"] = json!("subscription_cycle");
        renewal["amount_due"] = json!(29900);
        renewal["amount_paid"] = json!(29900);
        renewal["parent"] = json!({"subscription_details":{"subscription":subscription}});
        ledger.invoices.insert(0, renewal.clone());
        ledger.invoice_reads_fail = false;
        remote.subscription.latest_invoice = Some(serde_json::from_value(renewal).unwrap());
    }
    reconcile(&dashboard).await;
    assert_mismatch(&dashboard, quote, command, &invoice);
    assert_payment_preserved(&dashboard);
    reconcile(&dashboard).await;
    assert_mismatch(&dashboard, quote, command, &invoice);
    assert_payment_preserved(&dashboard);
    dashboard.close().await;
}

async fn reviewed_credit() -> (Dashboard, Browser, Value) {
    let (dashboard, payer) = machine_credit().await;
    let review = json(
        dashboard
            .request(
                Some(&payer),
                Method::GET,
                "/checkout/review?tier=business&start_blocks=0&machines=0",
                None,
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(review["today_cents"], "11500");
    let mut business = dashboard.fake.state.lock().unwrap().subscription.clone();
    business.items.data[0].id = "si_business".into();
    business.items.data[0].price.lookup_key = PriceCode::Business;
    business.items.data[0].price.id = "price_business".into();
    dashboard.fake.expect_subscription(
        SubscriptionRequest::Change(
            ChangeMode::Upgrade,
            vec![
                ItemChange::Remove("si_tier"),
                ItemChange::Add("price_business", 1),
            ],
        )
        .at(1795089600)
        .offset(-1000),
        SubscriptionOutcome::respond(business),
    );
    (dashboard, payer, review)
}

async fn pay(dashboard: &Dashboard, payer: &Browser, review: &Value) {
    let response = json(
        dashboard
            .request(
                Some(payer),
                Method::POST,
                "/billing/change",
                Some(json!({"quote_id":review["quote"]["id"]})),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(response["state"], "confirmed");
}

fn payment_identity(dashboard: &Dashboard) -> (Uuid, String) {
    let (markers, invoice) = {
        let remote = dashboard.fake.state.lock().unwrap();
        let markers = remote
            .requests
            .iter()
            .filter(|request| request.payment_marker())
            .cloned()
            .collect::<Vec<_>>();
        let invoice = remote.credits.as_ref().unwrap().invoices[0]["id"]
            .as_str()
            .unwrap()
            .to_owned();
        (markers, invoice)
    };
    assert_eq!(markers.len(), 1);
    let command = markers[0]
        .value("metadata[ciproxy_payment_id]")
        .unwrap()
        .parse()
        .unwrap();
    (command, invoice)
}

fn payment_work(dashboard: &Dashboard, command: Uuid) -> PaymentWork {
    dashboard
        .fixture
        .database
        .store
        .try_lock_billing(dashboard.fixture.workspace)
        .unwrap()
        .unwrap()
        .payment(command)
        .unwrap()
        .unwrap()
}

fn mismatch_alerts(dashboard: &Dashboard) -> Vec<BillingAlert> {
    let events = dashboard.alerts.0.lock().unwrap().clone();
    events
        .into_iter()
        .filter(|event| event.reason == AlertReason::InvoiceMismatch)
        .collect()
}

fn assert_mismatch(dashboard: &Dashboard, quote: Uuid, command: Uuid, invoice: &str) {
    let work = payment_work(dashboard, command);
    assert_eq!(work.quote.payment.id, quote);
    assert_eq!(work.quote.today_cents, 11500);
    assert_eq!(work.invoice_id.as_deref(), Some(invoice));
    assert_eq!(work.settlement, Settlement::Consumed);
    assert_eq!(work.restoration, None);
    assert!(!work.pending);
    assert_eq!(
        work.audit,
        InvoiceAudit::Mismatch {
            amount_cents: 12000,
            alerted: true,
        }
    );
    assert_eq!(
        mismatch_alerts(dashboard),
        vec![BillingAlert {
            workspace_id: dashboard.fixture.workspace,
            quote_id: quote,
            command_id: command,
            invoice_id: Some(invoice.into()),
            expected_cents: 11500,
            actual_cents: Some(12000),
            reason: AlertReason::InvoiceMismatch,
        }]
    );
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
    assert_eq!(
        record.state.paid_period_start,
        Some(at("2026-11-04T12:00:00Z"))
    );
    assert_eq!(
        record.state.paid_period_end,
        Some(at("2026-12-04T12:00:00Z"))
    );
    let (charges, balance, restorations, cancellation) = {
        let remote = dashboard.fake.state.lock().unwrap();
        let ledger = remote.credits.as_ref().unwrap();
        (
            ledger.charges.clone(),
            ledger.balance_cents,
            ledger.balance_transactions.clone(),
            remote.subscription.cancel_at_period_end,
        )
    };
    assert_eq!(charges, vec![12000]);
    assert_eq!(balance, 0);
    assert_eq!(restorations, Vec::<Value>::new());
    assert!(!cancellation);
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
