//! Request-driven pending credits and paid invoice amounts for Stripe HTTP tests.

use ciproxy_billing::catalog::PriceCode;
use ciproxy_billing::stripe::wire::types::{PendingUpdate, Subscription};
use serde_json::{Value, json};

use super::fake::State;
use super::fake_requests::Request;

pub(super) struct CreditLedger {
    pub(super) pending: Vec<Value>,
    pub(super) charges: Vec<u64>,
    pub(super) invoices: Vec<Value>,
    pub(super) monthly_cents: u64,
    pub(super) balance_cents: i64,
    pub(super) balance_transactions: Vec<Value>,
    pub(super) pending_payment: bool,
    pub(super) force_charge_difference: i64,
    pub(super) lose_restore_response: bool,
    pub(super) invoice_reads_fail: bool,
    pub(super) page_size: usize,
    pub(super) preview_lines: Vec<Value>,
    pub(super) fail_subscription_reads: usize,
}

impl CreditLedger {
    pub(super) fn new() -> Self {
        Self {
            pending: Vec::new(),
            charges: Vec::new(),
            invoices: Vec::new(),
            monthly_cents: 29900,
            balance_cents: 0,
            balance_transactions: Vec::new(),
            pending_payment: false,
            force_charge_difference: 0,
            lose_restore_response: false,
            invoice_reads_fail: false,
            page_size: 100,
            preview_lines: Vec::new(),
            fail_subscription_reads: 0,
        }
    }
}

pub(super) fn preview(state: &mut State, request: &Request) -> Value {
    let base = state
        .proration
        .as_ref()
        .unwrap()
        .amount(request, "subscription_details[proration_date]");
    let current = state.subscription.clone();
    let ledger = state.credits.as_mut().unwrap();
    let recurring = request.value("preview_mode") == Some("recurring");
    let pending: i64 = ledger
        .pending
        .iter()
        .filter(|item| item["invoice"].is_null())
        .map(|item| item["amount"].as_i64().unwrap())
        .sum();
    let amount = if recurring {
        ledger.monthly_cents
    } else {
        (base as i64 + pending + ledger.balance_cents).max(0) as u64
    };
    let mut lines = ledger
        .pending
        .iter()
        .filter(|item| item["invoice"].is_null())
        .map(|item| {
            let parent = if item["parent"].is_null() {
                json!({"type":"invoice_item_details","invoice_item_details":{
                "invoice_item":item["id"],"subscription":null,"proration":false}})
            } else {
                json!({"type":"subscription_item_details","subscription_item_details":{
                "invoice_item":item["id"],"subscription":current.id,
                "subscription_item":"si_machine","proration":true}})
            };
            json!({"id":format!("il_{}",item["id"].as_str().unwrap()),"amount":item["amount"],
            "currency":"usd","discount_amounts":[],"taxes":[],"parent":parent})
        })
        .collect::<Vec<_>>();
    lines.push(
        json!({"id":"il_new_proration","amount":base,"currency":"usd",
        "discount_amounts":[],"taxes":[],"parent":{"type":"subscription_item_details",
        "subscription_item_details":{"subscription":current.id,"subscription_item":"si_business",
        "invoice_item":null,"proration":true}}}),
    );
    if !recurring {
        ledger.preview_lines = lines.clone();
    }
    json!({"id":"upcoming_in_credit","currency":"usd","customer":current.customer,
        "parent":{"subscription_details":{"subscription":current.id}},"starting_balance":ledger.balance_cents,
        "amount_due":amount,"total":amount,"lines":{
            "data":lines.iter().take(ledger.page_size).collect::<Vec<_>>(),"has_more":lines.len()>ledger.page_size}})
}

pub(super) fn apply(state: &mut State, request: &Request, updated: &mut Subscription) {
    let Some(ledger) = &mut state.credits else {
        return;
    };
    let proration = state.proration.as_ref().unwrap();
    if request.value("proration_behavior") == Some("create_prorations") {
        let quantity = |value: &Subscription| {
            value
                .items
                .data
                .iter()
                .find(|item| item.price.lookup_key == PriceCode::Machine)
                .map_or(0, |item| item.quantity)
        };
        let removed = i64::from(quantity(updated)) - i64::from(quantity(&state.subscription));
        let amount = removed * 2000 * (proration.ends_at - proration.now)
            / (proration.ends_at - proration.starts_at);
        if amount != 0 {
            ledger.pending.push(
                json!({"id":"ii_machine_credit","amount":amount,"currency":"usd",
                "customer":state.subscription.customer,"invoice":null,"discountable":false,
                "parent":{"subscription_details":{"subscription":state.subscription.id}},
                "pricing":{"price_details":{"product":"prod_machine","price":"price_machine"}}}),
            );
        }
        return;
    }
    if request.value("proration_behavior") != Some("always_invoice") {
        return;
    }
    let offset: i64 = request
        .form
        .iter()
        .filter(|(key, _)| key.starts_with("add_invoice_items[") && key.ends_with("[unit_amount]"))
        .map(|(_, value)| value.parse::<i64>().unwrap())
        .sum();
    let total = proration.amount(request, "proration_date") as i64 + offset + ledger.balance_cents;
    let charged = (total + ledger.force_charge_difference).max(0) as u64;
    ledger.balance_cents = total.min(0);
    ledger.charges.push(charged);
    let marker = serde_json::to_value(&state.subscription.metadata.payment).unwrap();
    let id = format!(
        "in_credit_{}",
        state
            .subscription
            .metadata
            .payment
            .ciproxy_payment_id
            .unwrap()
            .simple()
    );
    if ledger.pending_payment {
        updated.items = state.subscription.items.clone();
        updated.pending_update = Some(PendingUpdate {
            expires_at: (proration.now + 23 * 3600).min(proration.ends_at),
        });
    }
    let invoice = json!({"id":id,"created":proration.now,"customer":updated.customer,"currency":"usd",
        "billing_reason":"subscription_update","status":if ledger.pending_payment {"open"} else {"paid"},
        "amount_due":charged,"amount_paid":if ledger.pending_payment {0} else {charged},
        "amount_remaining":if ledger.pending_payment {charged} else {0},
        "parent":{"subscription_details":{"subscription":updated.id,"metadata":marker}},
        "customer_email":"synthetic-private-canary","payment_method_details":{"synthetic_secret":"not-a-real-secret"},
        "hosted_invoice_url":"https://invoice.stripe.test/paid","next_payment_attempt":null});
    updated.latest_invoice = Some(serde_json::from_value(invoice.clone()).unwrap());
    state
        .raw_reads
        .insert(format!("/v1/invoices/{id}"), invoice.clone());
    ledger.invoices.push(invoice);
}
