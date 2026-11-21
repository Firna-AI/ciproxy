//! Minimal projections of the verified real Stripe recordings.

use std::collections::BTreeMap;

use ciproxy_billing::error::Result;
use ciproxy_billing::service::view::EventOutcome;
use ciproxy_store_interface::billing::state::BillingState;
use serde::Deserialize;
use serde_json::Value;

use super::database::Database;
use super::fake::Fake;
use super::fixture::{at, read_service, signature};

#[derive(Deserialize)]
pub(super) struct Recording {
    pub(super) initial_state: BillingState,
    pub(super) events: Vec<Value>,
    #[serde(skip)]
    pub(super) reads: BTreeMap<String, Value>,
}

impl Recording {
    pub(super) fn recovery() -> Self {
        Self::parse(
            include_str!("fixtures/live/recovery_events.json"),
            include_str!("fixtures/live/recovery_reads.json"),
        )
    }

    pub(super) fn final_failure() -> Self {
        Self::parse(
            include_str!("fixtures/live/final_events.json"),
            include_str!("fixtures/live/final_reads.json"),
        )
    }

    fn parse(events: &str, reads: &str) -> Self {
        let mut recording: Self = serde_json::from_str(events).unwrap();
        recording.reads = serde_json::from_str(reads).unwrap();
        recording
    }

    pub(super) fn snapshot(&self, id: &str, status: &str) -> Value {
        self.reads
            .values()
            .find(|body| body["id"] == id && body["status"] == status)
            .unwrap_or_else(|| panic!("missing recorded {id} with status {status}"))
            .clone()
    }

    pub(super) fn seed(&self) -> Database {
        let database = Database::new(self.initial_state.workspace_id);
        {
            let mut session = database
                .store
                .try_lock_billing(self.initial_state.workspace_id)
                .unwrap()
                .unwrap();
            session
                .commit(Some(&self.initial_state), None, true, false, at())
                .unwrap();
        }
        database
    }
}

pub(super) fn install(fake: &Fake, subscription: Value, invoices: &[Value]) {
    let mut remote = fake.state.lock().unwrap();
    remote.subscription = serde_json::from_value(subscription.clone()).unwrap();
    remote.raw_reads.clear();
    remote.raw_read_sequences.clear();
    remote.raw_reads.insert(
        format!("/v1/subscriptions/{}", subscription["id"].as_str().unwrap()),
        subscription,
    );
    for invoice in invoices {
        remote.raw_reads.insert(
            format!("/v1/invoices/{}", invoice["id"].as_str().unwrap()),
            invoice.clone(),
        );
    }
}

pub(super) fn deliver(database: &Database, fake: &Fake, event: &Value) -> EventOutcome {
    send(database, fake, event).unwrap()
}

pub(super) fn send(database: &Database, fake: &Fake, event: &Value) -> Result<EventOutcome> {
    let raw = serde_json::to_vec(event).unwrap();
    read_service(database, fake, at()).webhook(&signature(&raw, at()), &raw)
}

pub(super) fn saved(database: &Database, recording: &Recording) -> BillingState {
    database
        .store
        .billing(recording.initial_state.workspace_id)
        .unwrap()
        .state
}
