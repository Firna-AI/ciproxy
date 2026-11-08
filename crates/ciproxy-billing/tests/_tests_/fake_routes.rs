//! Fake HTTP responses preserve supplied outcomes after recording request checks.

use ciproxy_billing::stripe::wire::types::CheckoutSession;
use serde::Serialize;
use serde_json::{Value, json};
use url::form_urlencoded;

use super::fake::State;
use super::fake_requests::Request;
use super::fake_subscriptions::SubscriptionOutcome;

pub(super) fn handle(state: &mut State, request: &Request) -> (u16, String) {
    state.requests.push(request.clone());
    let path = request.path.split('?').next().unwrap_or("");
    if request.method == "POST"
        && path == "/v1/checkout/sessions"
        && let Some(status) = state.reject_next_checkout.take()
    {
        return (status, "{}".into());
    }
    let body = if request.method == "GET" {
        get(state, request, path)
    } else {
        post(state, request, path)
    };
    let (status, body) = body;
    if request.method == "DELETE" && state.lose_next_delete && status == 200 {
        state.lose_next_delete = false;
        return (503, "{}".into());
    }
    if request.method == "POST"
        && state.lose_next_post
        && !path.ends_with("/prices")
        && status == 200
    {
        state.lose_next_post = false;
        return (
            503,
            "{\"error\":{\"message\":\"hidden provider details\"}}".into(),
        );
    }
    (status, body)
}

fn get(state: &mut State, request: &Request, path: &str) -> (u16, String) {
    if let Some(body) = state
        .raw_read_sequences
        .get_mut(path)
        .and_then(|reads| reads.pop_front())
    {
        return (200, body.to_string());
    }
    if let Some(body) = state.raw_reads.get(path) {
        return (200, body.to_string());
    }
    match path {
        "/v1/prices" => {
            let query = request.path.split('?').nth(1).unwrap_or("");
            let code = form_urlencoded::parse(query.as_bytes())
                .find(|(key, _)| key == "lookup_keys[0]")
                .map(|(_, value)| value.into_owned())
                .unwrap_or_default();
            if state.create_prices && !state.rejected_price_codes.contains(&code) {
                return (200, json!({"data":[],"has_more":false}).to_string());
            }
            match price(&code) {
                Some(price) => (200, json!({"data":[price],"has_more":false}).to_string()),
                None => {
                    state.failure(format!("unexpected price lookup {code:?}"));
                    (400, "{}".into())
                }
            }
        }
        "/v1/checkout/sessions" => (
            200,
            json!({"data":state.checkout.iter().collect::<Vec<_>>(),"has_more":false}).to_string(),
        ),
        "/v1/checkout/sessions/cs_recorded" => encode(state, state.checkout.clone()),
        path if path == format!("/v1/subscriptions/{}", state.subscription.id) => {
            encode(state, state.subscription.clone())
        }
        path if path.starts_with("/v1/invoices/") => {
            let id = path.strip_prefix("/v1/invoices/").unwrap_or("");
            let invoice = state
                .invoices
                .iter()
                .find(|invoice| invoice.id == id)
                .or_else(|| {
                    state
                        .subscription
                        .latest_invoice
                        .as_ref()
                        .filter(|invoice| invoice.id == id)
                });
            let Some(invoice) = invoice else {
                return (404, "{}".into());
            };
            match serde_json::to_value(invoice) {
                Ok(mut body) => {
                    body["customer"] = json!(state.subscription.customer);
                    body["parent"] =
                        json!({"subscription_details":{"subscription":state.subscription.id}});
                    (200, body.to_string())
                }
                Err(error) => {
                    state.failure(format!("encode invoice: {error}"));
                    (500, "{}".into())
                }
            }
        }
        _ => {
            state.failure(format!(
                "unexpected fake HTTP request: {} {}",
                request.method, request.path
            ));
            (404, "{}".into())
        }
    }
}

fn post(state: &mut State, request: &Request, path: &str) -> (u16, String) {
    match path {
        "/v1/invoices/create_preview" => {
            if let Some(proration) = &state.proration {
                let amount = if request.value("preview_mode") == Some("recurring") {
                    31900
                } else {
                    proration.amount(request, "subscription_details[proration_date]")
                };
                return (
                    200,
                    json!({"currency":"usd", "customer":state.subscription.customer,
                    "parent":{"subscription_details":{"subscription":state.subscription.id}},
                    "amount_due":amount, "total":amount})
                    .to_string(),
                );
            }
            let Some((mut expected, reply)) = state.previews.pop_front() else {
                state.failure("unexpected invoice preview request".into());
                return (500, "{}".into());
            };
            let mut actual = request.form.clone();
            expected.sort();
            actual.sort();
            if expected != actual || request.key.is_some() {
                state.failure(format!(
                    "invoice preview fields: expected {expected:#?}, actual {actual:#?}"
                ));
            }
            (200, reply.to_string())
        }
        "/v1/prices" => {
            let code = request.value("lookup_key").unwrap_or_default();
            state.rejected_price_codes.push(code.into());
            match price(code) {
                Some(price) => (200, price.to_string()),
                None => {
                    state.failure(format!("unexpected price creation {code:?}"));
                    (400, "{}".into())
                }
            }
        }
        "/v1/billing_portal/configurations" => (200, json!({"id":"bpc_restricted"}).to_string()),
        "/v1/billing_portal/sessions" => (
            200,
            json!({"url":"https://billing.stripe.test/portal"}).to_string(),
        ),
        "/v1/checkout/sessions" => {
            let parsed =
                serde_json::from_str::<CheckoutSession>(include_str!("fixtures/checkout.json"));
            let mut checkout = match parsed {
                Ok(checkout) => checkout,
                Err(error) => {
                    state.failure(format!("parse Checkout fixture: {error}"));
                    return (500, "{}".into());
                }
            };
            let command = request
                .value("metadata[command_id]")
                .and_then(|value| value.parse().ok());
            if command.is_none() {
                state.failure("Checkout lacks a valid command identifier".into());
            }
            checkout.metadata.command_id = command;
            checkout.metadata.workspace_id = request
                .value("metadata[workspace_id]")
                .and_then(|value| value.parse().ok());
            state.checkout = Some(checkout.clone());
            encode(state, checkout)
        }
        path if path == format!("/v1/subscriptions/{}", state.subscription.id) => {
            match state.subscription_post(request) {
                Some(SubscriptionOutcome::Respond(subscription)) => {
                    if let Some(proration) = &mut state.proration {
                        proration
                            .charges
                            .push(proration.amount(request, "proration_date"));
                    }
                    state.subscription = *subscription;
                    if state.raw_reads.contains_key(path) {
                        state.raw_reads.insert(
                            path.into(),
                            serde_json::to_value(&state.subscription).unwrap(),
                        );
                    }
                    encode(state, state.subscription.clone())
                }
                Some(SubscriptionOutcome::Reject(status)) => (status, "{}".into()),
                None => (500, "{}".into()),
            }
        }
        _ => {
            state.failure(format!(
                "unexpected fake HTTP request: {} {}",
                request.method, request.path
            ));
            (404, "{}".into())
        }
    }
}

fn encode(state: &mut State, value: impl Serialize) -> (u16, String) {
    match serde_json::to_string(&value) {
        Ok(body) => (200, body),
        Err(error) => {
            state.failure(format!("encode fake response: {error}"));
            (500, "{}".into())
        }
    }
}

fn price(code: &str) -> Option<Value> {
    let (id, amount) = match code {
        "ciproxy_team_monthly_usd" => ("price_team", 4900),
        "ciproxy_business_monthly_usd" => ("price_business", 29900),
        "ciproxy_starts_monthly_usd" => ("price_starts", 1000),
        "ciproxy_machine_monthly_usd" => ("price_machine", 2000),
        _ => return None,
    };
    Some(
        json!({"id":id,"lookup_key":code,"currency":"usd","unit_amount":amount,
        "recurring":{"interval":"month","interval_count":1},"active":true}),
    )
}
