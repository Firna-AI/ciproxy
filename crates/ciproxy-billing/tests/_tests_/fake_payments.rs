//! Strict payment markers, pending-item ownership, and durable balance receipts.

use ciproxy_billing::stripe::wire::invoice::PaymentMetadata;
use serde_json::{Value, json};
use url::form_urlencoded;
use uuid::Uuid;

use super::fake::State;
use super::fake_requests::Request;

pub(super) fn get(state: &mut State, request: &Request, path: &str) -> Option<(u16, String)> {
    let query = form_urlencoded::parse(request.path.split('?').nth(1).unwrap_or("").as_bytes())
        .into_owned()
        .collect::<Vec<_>>();
    let value = |key: &str| {
        query
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.as_str())
    };
    let size = state
        .credits
        .as_ref()
        .map_or(100, |ledger| ledger.page_size);
    let body = if path == "/v1/invoiceitems" {
        if value("customer") != Some(state.subscription.customer.as_str())
            || value("pending") != Some("true")
        {
            state.failure("pending item query has the wrong customer or pending flag".into());
            return Some((400, "{}".into()));
        }
        let items = state.credits.as_ref().map_or_else(Vec::new, |ledger| {
            ledger
                .pending
                .iter()
                .filter(|item| item["invoice"].is_null())
                .cloned()
                .collect()
        });
        page(items, value("starting_after"), size)
    } else if let Some(id) = path.strip_prefix("/v1/invoiceitems/") {
        return Some(
            state
                .credits
                .as_ref()
                .and_then(|ledger| ledger.pending.iter().find(|item| item["id"] == id))
                .map_or((404, "{}".into()), |item| (200, item.to_string())),
        );
    } else if path == "/v1/invoices/upcoming_in_credit/lines" {
        page(
            state.credits.as_ref().unwrap().preview_lines.clone(),
            value("starting_after"),
            size,
        )
    } else if path == "/v1/invoices" {
        page(
            state
                .credits
                .as_ref()
                .map_or_else(Vec::new, |ledger| ledger.invoices.clone()),
            value("starting_after"),
            size,
        )
    } else if path == format!("/v1/customers/{}", state.subscription.customer) {
        let amount = state
            .credits
            .as_ref()
            .map_or(0, |ledger| ledger.balance_cents);
        json!({"id":state.subscription.customer,"balance":amount,"invoice_credit_balance":{"usd":amount}})
    } else if path.ends_with("/balance_transactions") {
        page(
            state
                .credits
                .as_ref()
                .map_or_else(Vec::new, |ledger| ledger.balance_transactions.clone()),
            value("starting_after"),
            size,
        )
    } else {
        let id = path.strip_prefix("/v1/prices/")?;
        let product = id
            .strip_prefix("price_")
            .map(|name| format!("prod_{name}"))?;
        json!({"id":id,"product":product})
    };
    Some((200, body.to_string()))
}

pub(super) fn post(state: &mut State, request: &Request, path: &str) -> Option<(u16, String)> {
    if path.starts_with("/v1/subscriptions/") && request.payment_marker() {
        let quote = request
            .value("metadata[ciproxy_quote_id]")
            .and_then(|value| value.parse::<Uuid>().ok());
        let command = request
            .value("metadata[ciproxy_payment_id]")
            .and_then(|value| value.parse::<Uuid>().ok());
        if quote.is_none()
            || command.is_none()
            || request.key != command.map(|id| format!("ciproxy:quote-marker:{id}"))
        {
            state.failure("invalid paid quote marker or idempotency key".into());
            return Some((400, "{}".into()));
        }
        state.subscription.metadata.payment = PaymentMetadata {
            ciproxy_quote_id: quote,
            ciproxy_payment_id: command,
        };
        if let Some(raw) = state.raw_reads.get_mut(path) {
            raw["metadata"]["ciproxy_quote_id"] = json!(quote);
            raw["metadata"]["ciproxy_payment_id"] = json!(command);
        }
        return Some((200, serde_json::to_string(&state.subscription).unwrap()));
    }
    if path.ends_with("/balance_transactions") {
        let amount = request
            .value("amount")
            .and_then(|value| value.parse::<i64>().ok());
        let command = request
            .value("metadata[ciproxy_payment_id]")
            .and_then(|value| value.parse::<Uuid>().ok());
        if request.form.len() != 3
            || amount.is_none()
            || request.value("currency") != Some("usd")
            || request.key != command.map(|id| format!("ciproxy:credit-restore:{id}"))
        {
            state.failure("invalid signed balance restoration request".into());
            return Some((400, "{}".into()));
        }
        let ledger = state.credits.as_mut().unwrap();
        let amount = amount.unwrap();
        ledger.balance_cents += amount;
        let receipt = json!({"id":format!("cbtxn_{}",ledger.balance_transactions.len()+1),
            "customer":state.subscription.customer,"amount":amount,"currency":"usd",
            "metadata":{"ciproxy_payment_id":command}});
        ledger.balance_transactions.push(receipt.clone());
        if ledger.lose_restore_response {
            ledger.lose_restore_response = false;
            return Some((503, "{}".into()));
        }
        return Some((200, receipt.to_string()));
    }
    if let Some(id) = path.strip_prefix("/v1/invoiceitems/")
        && request.method == "DELETE"
    {
        let ledger = state.credits.as_mut().unwrap();
        let Some(index) = ledger.pending.iter().position(|item| item["id"] == id) else {
            return Some((404, "{}".into()));
        };
        if !ledger.pending[index]["invoice"].is_null() {
            return Some((400, "{}".into()));
        }
        ledger.pending.remove(index);
        return Some((200, json!({"id":id,"deleted":true}).to_string()));
    }
    None
}

fn page(items: Vec<Value>, after: Option<&str>, size: usize) -> Value {
    let start = after.map_or(0, |id| {
        items.iter().position(|item| item["id"] == id).unwrap() + 1
    });
    let end = (start + size).min(items.len());
    json!({"data":items[start..end],"has_more":end<items.len()})
}
