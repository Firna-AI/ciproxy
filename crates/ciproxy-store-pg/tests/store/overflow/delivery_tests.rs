//! Callback scope, durable key retention, and concurrent session exclusion.

use chrono::Duration;
use ciproxy_store_interface::overflow::record::{EncryptedLaunch, StartPhase};
use ciproxy_store_interface::secrets::TokenHash;
use uuid::Uuid;

use super::super::support::{Database, at};
use super::support::setup;

#[test]
fn ambiguous_start_retains_its_old_key_through_rotation_and_disconnect() {
    let database = Database::new();
    let (workspace_id, account, mut record) = setup(&database);
    let hash = TokenHash::new(vec![1, 2, 3]);
    {
        let mut session = database
            .store
            .try_lock_overflow(workspace_id)
            .unwrap()
            .unwrap();
        assert!(
            session
                .reserve_start(&record, &EncryptedLaunch(vec![91, 92]), &hash)
                .unwrap()
        );
        let mut replacement = account.clone();
        replacement.account.id = Uuid::new_v4();
        replacement.account.connected_at = at() + Duration::seconds(1);
        session.connect(&replacement, 300_000_000).unwrap();
        session.disconnect(at() + Duration::seconds(2)).unwrap();
        session.collect(at() - Duration::days(90)).unwrap();
        assert_eq!(
            session.snapshot().unwrap().accounts,
            vec![ciproxy_store_interface::overflow::store::AccountRecord {
                account: ciproxy_store_interface::boat::BoatAccount {
                    retired_at: Some(at() + Duration::seconds(1)),
                    ..account.account.clone()
                },
                ..account.clone()
            }]
        );
        assert_eq!(
            session.encrypted_launch(record.delivery_id).unwrap(),
            Some(EncryptedLaunch(vec![91, 92]))
        );
    }
    assert_eq!(
        database
            .store
            .encrypted_boat_key(workspace_id, account.account.id)
            .unwrap(),
        Some(account.key)
    );
    record.phase = StartPhase::Ended;
    record.ends_at = Some(at() + Duration::hours(6));
    {
        let mut session = database.store.lock_overflow(workspace_id).unwrap();
        session.save_start(&record).unwrap();
        session.collect(at() - Duration::days(90)).unwrap();
        assert_eq!(session.snapshot().unwrap().accounts, []);
        assert_eq!(session.encrypted_launch(record.delivery_id).unwrap(), None);
    }
    assert_eq!(
        database
            .store
            .encrypted_boat_key(workspace_id, account.account.id)
            .unwrap(),
        None
    );
}

#[test]
fn callback_claim_is_scoped_one_use_and_durable_before_deletion() {
    let database = Database::new();
    let (workspace_id, _, mut record) = setup(&database);
    let other = database.workspace();
    let hash = TokenHash::new(vec![5, 6, 7]);
    let mut session = database.store.lock_overflow(workspace_id).unwrap();
    assert!(
        session
            .reserve_start(&record, &EncryptedLaunch(vec![71]), &hash)
            .unwrap()
    );
    record.sandbox_id = Some("bx_23456789".to_owned());
    record.phase = StartPhase::Running;
    session.save_start(&record).unwrap();
    assert_eq!(
        database.store.overflow_token_workspace(&hash).unwrap(),
        Some(workspace_id)
    );
    let mut other_session = database.store.lock_overflow(other.id).unwrap();
    assert_eq!(
        other_session
            .accept_end(&hash, record.runner_id, at())
            .unwrap(),
        None
    );
    assert_eq!(
        session.accept_end(&hash, Uuid::new_v4(), at()).unwrap(),
        None
    );
    let accepted = session
        .accept_end(&hash, record.runner_id, at() + Duration::hours(7))
        .unwrap()
        .unwrap();
    assert_eq!(accepted.phase, StartPhase::Deleting);
    assert_eq!(accepted.reported_at, Some(at() + Duration::hours(7)));
    assert_eq!(session.snapshot().unwrap().starts, vec![accepted]);
    assert_eq!(
        session
            .accept_end(&hash, record.runner_id, at() + Duration::hours(8))
            .unwrap(),
        None
    );
    assert_eq!(
        database
            .store
            .runners(workspace_id, record.organization_id)
            .unwrap()[0]
            .size
            .disk_gb,
        111
    );
}

#[test]
fn overflow_sessions_exclude_each_other_and_reservation_rechecks_tenant_authority() {
    let database = Database::new();
    let (workspace_id, _, record) = setup(&database);
    let mut session = database.store.lock_overflow(workspace_id).unwrap();
    assert!(
        database
            .store
            .try_lock_overflow(workspace_id)
            .unwrap()
            .is_none()
    );
    let other = database.workspace();
    let mut other_session = database.store.lock_overflow(other.id).unwrap();
    assert!(
        !other_session
            .reserve_start(&record, &EncryptedLaunch(vec![1]), &TokenHash::new(vec![2]))
            .unwrap()
    );
    database
        .store
        .set_app_suspended(workspace_id, record.organization_id, true)
        .unwrap();
    assert!(
        !session
            .reserve_start(&record, &EncryptedLaunch(vec![1]), &TokenHash::new(vec![2]))
            .unwrap()
    );
    database
        .store
        .set_app_suspended(workspace_id, record.organization_id, false)
        .unwrap();
    session.disconnect(at()).unwrap();
    assert!(
        !session
            .reserve_start(&record, &EncryptedLaunch(vec![1]), &TokenHash::new(vec![2]))
            .unwrap()
    );
}
