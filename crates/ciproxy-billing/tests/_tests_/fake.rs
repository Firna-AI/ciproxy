//! A real HTTP Stripe double with mandatory subscription exchanges.

use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use ciproxy_billing::stripe::wire::invoice::Invoice;
use ciproxy_billing::stripe::wire::types::{CheckoutSession, Subscription};
use serde_json::Value;
use tiny_http::{Response, Server};
use url::form_urlencoded;

use super::fake_requests::Request;
use super::fake_routes::handle;
use super::fake_subscriptions::{SubscriptionExchange, SubscriptionOutcome, SubscriptionRequest};

pub(super) struct State {
    pub(super) checkout: Option<CheckoutSession>,
    pub(super) subscription: Subscription,
    pub(super) invoices: Vec<Invoice>,
    pub(super) raw_reads: BTreeMap<String, Value>,
    pub(super) raw_read_sequences: BTreeMap<String, VecDeque<Value>>,
    pub(super) requests: Vec<Request>,
    pub(super) previews: VecDeque<(Vec<(String, String)>, Value)>,
    pub(super) lose_next_post: bool,
    pub(super) lose_next_delete: bool,
    pub(super) reject_next_checkout: Option<u16>,
    pub(super) create_prices: bool,
    pub(super) rejected_price_codes: Vec<String>,
    exchanges: VecDeque<SubscriptionExchange>,
    failures: Vec<String>,
}

impl State {
    pub(super) fn expect_subscription(
        &mut self,
        expected: SubscriptionRequest,
        outcome: SubscriptionOutcome,
    ) {
        self.exchanges
            .push_back(SubscriptionExchange::new(expected, outcome));
    }

    pub(super) fn failure(&mut self, message: String) {
        self.failures.push(message);
    }

    pub(super) fn subscription_post(&mut self, request: &Request) -> Option<SubscriptionOutcome> {
        let Some(exchange) = self.exchanges.pop_front() else {
            self.failure(format!(
                "unexpected subscription request with no pair: {request:#?}"
            ));
            return None;
        };
        let (errors, outcome) = exchange.consume(request);
        self.failures.extend(errors);
        Some(outcome)
    }
}

#[derive(Clone)]
pub(super) struct Probe(Arc<Mutex<State>>);

impl Probe {
    pub(super) fn check(&self) {
        let (failures, versions) = {
            let state = self.0.lock().unwrap();
            (
                state.failures.clone(),
                state
                    .requests
                    .iter()
                    .map(|request| request.stripe_version.clone())
                    .collect::<Vec<_>>(),
            )
        };
        for version in versions {
            assert_eq!(version.as_deref(), Some("2026-10-28.endive"));
        }
        assert!(
            failures.is_empty(),
            "Stripe request mismatch on test thread:\n{}",
            failures.join("\n")
        );
    }
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
            invoices: Vec::new(),
            raw_reads: BTreeMap::new(),
            raw_read_sequences: BTreeMap::new(),
            requests: Vec::new(),
            previews: VecDeque::new(),
            lose_next_post: false,
            lose_next_delete: false,
            reject_next_checkout: None,
            create_prices: false,
            rejected_price_codes: Vec::new(),
            exchanges: VecDeque::new(),
            failures: Vec::new(),
        }));
        let worker_server = server.clone();
        let worker_state = state.clone();
        let thread = thread::spawn(move || {
            while let Ok(mut incoming) = worker_server.recv() {
                let mut body = String::new();
                let read = incoming.as_reader().read_to_string(&mut body);
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
                    stripe_version: incoming
                        .headers()
                        .iter()
                        .find(|header| header.field.equiv("Stripe-Version"))
                        .map(|header| header.value.as_str().to_owned()),
                };
                let (status, response) = {
                    let mut state = worker_state.lock().unwrap();
                    if let Err(error) = read {
                        state.failure(format!("read fake HTTP request: {error}"));
                        (400, "{}".into())
                    } else {
                        handle(&mut state, &request)
                    }
                };
                if let Err(error) =
                    incoming.respond(Response::from_string(response).with_status_code(status))
                {
                    worker_state
                        .lock()
                        .unwrap()
                        .failure(format!("send fake HTTP response: {error}"));
                }
            }
        });
        Self {
            state,
            url,
            server,
            thread: Some(thread),
        }
    }

    pub(super) fn expect_subscription(
        &self,
        expected: SubscriptionRequest,
        outcome: SubscriptionOutcome,
    ) {
        self.state
            .lock()
            .unwrap()
            .expect_subscription(expected, outcome);
    }

    pub(super) fn probe(&self) -> Probe {
        Probe(self.state.clone())
    }

    pub(super) fn posts(&self, path: &str) -> Vec<Request> {
        self.probe().check();
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
        let worker_failed = self
            .thread
            .take()
            .is_some_and(|thread| thread.join().is_err());
        let mut failures = {
            let state = self.state.lock().unwrap();
            let mut failures = state.failures.clone();
            failures.extend(state.exchanges.iter().map(|exchange| {
                format!(
                    "unconsumed subscription request pair: {}",
                    exchange.describe()
                )
            }));
            if !state.previews.is_empty() {
                failures.push("unconsumed invoice preview request pair".into());
            }
            failures
        };
        if worker_failed {
            failures.push("fake HTTP worker failed".into());
        }
        if thread::panicking() {
            if !failures.is_empty() {
                eprintln!(
                    "Stripe fake verification on test thread:\n{}",
                    failures.join("\n")
                );
            }
        } else {
            self.probe().check();
            assert!(
                failures.is_empty(),
                "Stripe fake verification on test thread:\n{}",
                failures.join("\n")
            );
        }
    }
}

pub(super) fn field(request: &Request, name: &str) -> String {
    request.value(name).unwrap().to_owned()
}
