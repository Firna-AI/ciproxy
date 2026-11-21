//! Removal revokes a connection that has not finished provider verification.

use std::sync::{Arc, Mutex};

use ciproxy_overflow::api::Overflow;
use ciproxy_overflow::error::Error;
use ciproxy_overflow::provider::BoatAccountApiMock;
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::store::Store;
use sandbox_interface::SandboxAccountCapacityMock;
use unimock::{MockFn, Unimock, matching};

use super::fixtures::{at, capabilities, facts, id};
use super::harness::{Harness, Runtime};

#[tokio::test]
async fn last_app_removal_during_verification_refuses_connection_without_an_active_key() {
    revoked_connection(false).await;
}

#[tokio::test]
async fn removal_and_reinstall_during_verification_cannot_restore_the_old_connection() {
    revoked_connection(true).await;
}

async fn revoked_connection(reinstall: bool) {
    let pending = Arc::new(Mutex::new(None::<Arc<dyn Store>>));
    let verifying = pending.clone();
    let backend = capabilities(Unimock::new((
        BoatAccountApiMock::account
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                let store = verifying.lock().unwrap().take().unwrap();
                let organization = store.organization(id(1), id(3)).unwrap().unwrap();
                store.record_app_removal(id(1), id(3), at()).unwrap();
                if reinstall {
                    store.install_organization(&organization).unwrap();
                }
                Ok(facts())
            }))
            .once(),
        SandboxAccountCapacityMock::account_capacity
            .some_call(matching!())
            .returns(Ok(vec![sandbox_boat::catalogue::offer("large").unwrap()]))
            .once(),
    )));
    let harness = Harness::new(backend, (), Runtime::Connect);
    *pending.lock().unwrap() = Some(harness.database.store.clone());
    let outcome = harness
        .service
        .connect(
            id(1),
            id(8),
            Secret::new("synthetic-pending-key"),
            "personal".to_owned(),
            20_000_000,
        )
        .await;
    assert!(
        matches!(outcome, Err(Error::ConnectionNotPermitted)),
        "revoked connection must fail: {outcome:?}"
    );
    assert_eq!(harness.snapshot().accounts, []);
    assert_eq!(
        harness
            .database
            .store
            .encrypted_boat_key(id(1), id(8))
            .unwrap(),
        None
    );
    assert_eq!(
        harness
            .database
            .store
            .workspace(id(1))
            .unwrap()
            .unwrap()
            .delete_after
            .is_none(),
        reinstall
    );
    harness
        .database
        .store
        .record_app_removal(id(1), id(3), at())
        .unwrap();
    let no_app = harness
        .service
        .connect(
            id(1),
            id(8),
            Secret::new("synthetic-no-app-key"),
            "personal".to_owned(),
            20_000_000,
        )
        .await;
    assert!(matches!(no_app, Err(Error::ConnectionNotPermitted)));
    assert_eq!(harness.snapshot().accounts, []);
}
