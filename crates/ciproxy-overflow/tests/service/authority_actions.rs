//! Durable authority changes and public reconciliation entry points.

use ciproxy_overflow::api::Overflow;
use ciproxy_protocol::secret::Secret;

use super::authority_support::disable;
use super::fixtures::{at, id, request};
use super::harness::Harness;

#[derive(Clone, Copy, Debug)]
pub(super) enum Withdrawal {
    Disconnect,
    Replacement,
    Disabled,
    Rejected,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Entry {
    Resume,
    Duplicate,
    Cleanup,
}

pub(super) async fn withdraw(harness: &Harness, withdrawal: Withdrawal) {
    match withdrawal {
        Withdrawal::Disconnect => harness.service.disconnect(id(1)).await.unwrap(),
        Withdrawal::Replacement => {
            harness
                .service
                .connect(
                    id(1),
                    id(44),
                    Secret::new("replacement-key"),
                    "personal".to_owned(),
                    200_000_000,
                )
                .await
                .unwrap();
        }
        Withdrawal::Disabled => disable(&harness.database.store),
        Withdrawal::Rejected => harness
            .database
            .store
            .reject_boat_key(id(1), id(2), at())
            .unwrap(),
    }
}

pub(super) async fn reconcile(harness: &Harness, entry: Entry) {
    match entry {
        Entry::Resume => {
            harness.service.resume(id(1), id(11)).await.unwrap();
        }
        Entry::Duplicate => {
            harness.service.start(id(1), request()).await.unwrap();
        }
        Entry::Cleanup => harness.service.cleanup(id(1)).await.unwrap(),
    }
}
