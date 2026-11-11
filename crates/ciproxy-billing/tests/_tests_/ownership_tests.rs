//! Billing isolation and atomic receipt rollback through PostgreSQL.

use chrono::Utc;
use ciproxy_billing::access::{BillingAccess, Role};
use ciproxy_billing::error::Error;
use ciproxy_store_interface::error;
use ciproxy_store_interface::workspaces::Workspace;
use uuid::Uuid;

use super::database::Database;
use super::fake::Fake;
use super::fixture::{
    access, at, idle_service, seed_checkout, seed_paid, state, subscription, workspace,
};
use super::quote_fixture::quote;

#[test]
fn failed_state_commit_rolls_back_event_receipt_and_preserves_both_workspaces() {
    let database = Database::new(workspace());
    seed_paid(&database);
    let before = state(&database);
    let other = Uuid::new_v4();
    database
        .store
        .create_workspace(&Workspace {
            id: other,
            name: "Other".into(),
            delete_after: None,
        })
        .unwrap();
    let mut lock = database.store.try_lock_billing(other).unwrap().unwrap();
    let free = lock.record().unwrap().state;
    let mut conflicting = free.clone();
    conflicting.subscription_id = Some("sub_recorded".into());
    assert!(matches!(
        lock.commit(Some(&conflicting), Some("evt_atomic"), false, false, at()),
        Err(error::Error::Conflict)
    ));
    assert!(!lock.event_applied("evt_atomic").unwrap());
    assert_eq!(lock.record().unwrap().state, free);
    assert_eq!(state(&database), before);
    conflicting.subscription_id = Some("sub_other".into());
    lock.commit(Some(&conflicting), Some("evt_atomic"), false, false, at())
        .unwrap();
    assert!(lock.event_applied("evt_atomic").unwrap());
    assert_eq!(
        lock.record().unwrap().state.subscription_id,
        Some("sub_other".into())
    );
    lock.commit(Some(&free), Some("evt_atomic"), false, false, at())
        .unwrap();
    assert_eq!(
        lock.record().unwrap().state.subscription_id,
        Some("sub_other".into())
    );
}

#[test]
fn return_session_from_another_workspace_cannot_apply_a_paid_tier() {
    let database = Database::new(workspace());
    let other = Uuid::new_v4();
    database
        .store
        .create_workspace(&Workspace {
            id: other,
            name: "Other".into(),
            delete_after: None,
        })
        .unwrap();
    let before = database.store.billing(other).unwrap();
    let fake = Fake::new(subscription());
    seed_checkout(&database, &fake);
    let service = idle_service(&database, &fake);
    assert!(matches!(
        service.reconcile_checkout(
            BillingAccess {
                workspace_id: other,
                role: Role::Admin
            },
            "cs_recorded"
        ),
        Err(Error::Ownership)
    ));
    assert_eq!(database.store.billing(other).unwrap(), before);
    assert_eq!(state(&database).subscription_id, None);
    seed_paid(&database);
    let mut wrong = subscription();
    wrong.metadata.workspace_id = Some(other);
    {
        let mut remote = fake.state.lock().unwrap();
        remote.subscription = wrong;
    }
    assert!(matches!(
        service.change(access(), quote(&database, &fake, Utc::now())),
        Err(Error::Ownership)
    ));
    assert_eq!(fake.subscription_mutations().len(), 0);
    assert!(matches!(
        service.move_to_free(access()),
        Err(Error::Ownership)
    ));
    assert_eq!(fake.subscription_mutations().len(), 0);
}
