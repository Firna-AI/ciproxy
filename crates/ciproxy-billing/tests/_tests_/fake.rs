//! A real HTTP Stripe double with scripted provider outcomes.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use ciproxy_billing::stripe::wire::types::{CheckoutSession, Subscription};
use serde_json::json;
use tiny_http::{Response, Server};
use url::form_urlencoded;

#[derive(Clone, Debug)]
pub(super) struct Request {
    pub(super) method: String,
    pub(super) path: String,
    pub(super) form: Vec<(String, String)>,
    pub(super) key: Option<String>,
}

pub(super) struct State {
    pub(super) checkout: Option<CheckoutSession>,
    pub(super) subscription: Subscription,
    pub(super) next_subscriptions: VecDeque<Subscription>,
    pub(super) requests: Vec<Request>,
    pub(super) lose_next_post: bool,
    pub(super) reject_next_mutation: Option<u16>,
    pub(super) create_prices: bool,
    pub(super) rejected_price_codes: Vec<String>,
}

pub(super) struct Fake {
    pub(super) state: Arc<Mutex<State>>,
    pub(super) url: String,
    server: Arc<Server>,
    thread: Option<JoinHandle<()>>,
}

impl Fake {
    pub(super) fn new(subscription: Subscription) -> Self {
        let server = Arc::new(Server::http("127.0.0.1:0").unwrap());
        let url = format!("http://{}/", server.server_addr());
        let state = Arc::new(Mutex::new(State {
            checkout: None,
            subscription,
            next_subscriptions: VecDeque::new(),
            requests: Vec::new(),
            lose_next_post: false,
            reject_next_mutation: None,
            create_prices: false,
            rejected_price_codes: Vec::new(),
        }));
        let worker_server = server.clone();
        let worker_state = state.clone();
        let thread = thread::spawn(move || {
            while let Ok(mut incoming) = worker_server.recv() {
                let mut body = String::new();
                incoming.as_reader().read_to_string(&mut body).unwrap();
                let request = Request {
                    method: incoming.method().to_string(),
                    path: incoming.url().to_owned(),
                    form: form_urlencoded::parse(body.as_bytes())
                        .into_owned()
                        .collect(),
                    key: incoming
                        .headers()
                        .iter()
                        .find(|header| header.field.equiv("Idempotency-Key"))
                        .map(|header| header.value.as_str().to_owned()),
                };
                assert_eq!(
                    incoming
                        .headers()
                        .iter()
                        .find(|header| header.field.equiv("Stripe-Version"))
                        .unwrap()
                        .value
                        .as_str(),
                    "2026-10-28.endive"
                );
                let (status, response) = handle(&mut worker_state.lock().unwrap(), &request);
                incoming
                    .respond(Response::from_string(response).with_status_code(status))
                    .unwrap();
            }
        });
        Self {
            state,
            url,
            server,
            thread: Some(thread),
        }
    }

    pub(super) fn posts(&self, path: &str) -> Vec<Request> {
        self.state
            .lock()
            .unwrap()
            .requests
            .iter()
            .filter(|request| request.method == "POST" && request.path == path)
            .cloned()
            .collect()
    }
}

impl Drop for Fake {
    fn drop(&mut self) {
        self.server.unblock();
        self.thread.take().unwrap().join().unwrap();
    }
}

fn handle(state: &mut State, request: &Request) -> (u16, String) {
    state.requests.push(request.clone());
    let path = request.path.split('?').next().unwrap();
    if request.method == "POST"
        && matches!(
            path,
            "/v1/checkout/sessions" | "/v1/subscriptions/sub_recorded"
        )
        && let Some(status) = state.reject_next_mutation.take()
    {
        return (status, "{}".into());
    }
    let body = if request.method == "GET" {
        match path {
            "/v1/prices" => {
                let query = request.path.split('?').nth(1).unwrap();
                let code = form_urlencoded::parse(query.as_bytes())
                    .find(|(key, _)| key == "lookup_keys[0]")
                    .unwrap()
                    .1
                    .into_owned();
                if state.create_prices && !state.rejected_price_codes.contains(&code) {
                    json!({"data":[], "has_more":false}).to_string()
                } else {
                    json!({"data":[price(&code)], "has_more":false}).to_string()
                }
            }
            "/v1/checkout/sessions" => {
                json!({"data":state.checkout.iter().collect::<Vec<_>>(),"has_more":false})
                    .to_string()
            }
            "/v1/checkout/sessions/cs_recorded" => {
                serde_json::to_string(state.checkout.as_ref().unwrap()).unwrap()
            }
            "/v1/subscriptions/sub_recorded" => serde_json::to_string(&state.subscription).unwrap(),
            "/v1/invoices/in_recorded" => {
                json!({"parent":{"subscription_details":{"subscription":"sub_recorded"}}})
                    .to_string()
            }
            _ => return (404, "{}".into()),
        }
    } else {
        match path {
            "/v1/prices" => {
                let code = field(request, "lookup_key");
                state.rejected_price_codes.push(code.clone());
                price(&code).to_string()
            }
            "/v1/billing_portal/configurations" => json!({"id":"bpc_restricted"}).to_string(),
            "/v1/billing_portal/sessions" => {
                json!({"url":"https://billing.stripe.test/portal"}).to_string()
            }
            "/v1/checkout/sessions" => {
                let mut checkout: CheckoutSession =
                    serde_json::from_str(include_str!("fixtures/checkout.json")).unwrap();
                checkout.metadata.command_id =
                    Some(field(request, "metadata[command_id]").parse().unwrap());
                state.checkout = Some(checkout.clone());
                serde_json::to_string(&checkout).unwrap()
            }
            "/v1/subscriptions/sub_recorded" => {
                state.subscription = state
                    .next_subscriptions
                    .pop_front()
                    .expect("script a literal Stripe update outcome");
                serde_json::to_string(&state.subscription).unwrap()
            }
            _ => return (404, "{}".into()),
        }
    };
    if request.method == "POST" && state.lose_next_post && !path.ends_with("/prices") {
        state.lose_next_post = false;
        return (
            503,
            "{\"error\":{\"message\":\"hidden provider details\"}}".into(),
        );
    }
    (200, body)
}

pub(super) fn field(request: &Request, name: &str) -> String {
    request
        .form
        .iter()
        .find(|(key, _)| key == name)
        .unwrap()
        .1
        .clone()
}

fn price(code: &str) -> serde_json::Value {
    let (id, amount) = match code {
        "ciproxy_team_monthly_usd" => ("price_team", 4900),
        "ciproxy_business_monthly_usd" => ("price_business", 29900),
        "ciproxy_starts_monthly_usd" => ("price_starts", 1000),
        "ciproxy_machine_monthly_usd" => ("price_machine", 2000),
        _ => panic!("unexpected catalog lookup"),
    };
    json!({"id":id,"lookup_key":code,"currency":"usd","unit_amount":amount,
        "recurring":{"interval":"month","interval_count":1},"active":true})
}
