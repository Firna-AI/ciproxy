//! Durable key retention and concurrent delivery session exclusion.

use chrono::Duration;
use ciproxy_store_interface::overflow::admission::{ReservationBudget, ReservationOutcome};
use ciproxy_store_interface::overflow::record::{EncryptedLaunch, StartPhase};
use ciproxy_store_interface::secrets::TokenHash;
use diesel::prelude::*;
use diesel::sql_types;
use rust_decimal::Decimal;
use uuid::Uuid;

use super::super::support::{Database, at};
use super::authority_support::assert_label_and_binding_liveness;
use super::support::{reserve, setup};

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
        assert_eq!(
            reserve(
                &mut *session,
                &record,
                &EncryptedLaunch(vec![91, 92]),
                &hash,
                &ReservationBudget {
                    current_month_usd: Decimal::new(462, 3),
                    next_month_usd: None,
                },
            )
            .unwrap(),
            ReservationOutcome::Reserved
        );
        let mut replacement = account.clone();
        replacement.account.id = Uuid::new_v4();
        replacement.account.connected_at = at() + Duration::seconds(1);
        let grant = session
            .begin_connection(replacement.account.id)
            .unwrap()
            .unwrap();
        assert!(
            session
                .complete_connection(grant, &replacement, 300_000_000)
                .unwrap()
        );
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
        assert_eq!(
            database
                .store
                .accept_end(
                    workspace_id,
                    &hash,
                    record.runner_id,
                    at() + Duration::hours(7)
                )
                .unwrap(),
            None
        );
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
fn overflow_sessions_exclude_each_other_and_reservation_rechecks_tenant_authority() {
    let mut database = Database::new();
    let (workspace_id, account, record) = setup(&database);
    let mut session = database.store.lock_overflow(workspace_id).unwrap();
    assert!(
        database
            .store
            .try_lock_overflow(workspace_id)
            .unwrap()
            .is_none()
    );
    let other = database.workspace();
    database
        .store
        .save_settings(other.id, &database.store.settings(workspace_id).unwrap())
        .unwrap();
    let mut other_session = database.store.lock_overflow(other.id).unwrap();
    assert_eq!(
        reserve(
            &mut *other_session,
            &record,
            &EncryptedLaunch(vec![1]),
            &TokenHash::new(vec![2]),
            &ReservationBudget {
                current_month_usd: Decimal::new(462, 3),
                next_month_usd: None,
            },
        )
        .unwrap(),
        ReservationOutcome::NotPermitted
    );
    database
        .store
        .set_app_suspended(workspace_id, record.organization_id, true)
        .unwrap();
    assert_eq!(
        reserve(
            &mut *session,
            &record,
            &EncryptedLaunch(vec![1]),
            &TokenHash::new(vec![2]),
            &ReservationBudget {
                current_month_usd: Decimal::new(462, 3),
                next_month_usd: None,
            },
        )
        .unwrap(),
        ReservationOutcome::NotPermitted
    );
    database
        .store
        .set_app_suspended(workspace_id, record.organization_id, false)
        .unwrap();
    let mut retained = record.clone();
    retained.delivery_id = Uuid::new_v4();
    retained.runner_id = Uuid::new_v4();
    let budget = ReservationBudget {
        current_month_usd: Decimal::new(462, 3),
        next_month_usd: None,
    };
    assert_eq!(
        reserve(
            &mut *session,
            &retained,
            &EncryptedLaunch(vec![1]),
            &TokenHash::new(vec![4]),
            &budget
        )
        .unwrap(),
        ReservationOutcome::Reserved
    );
    assert_eq!(
        session.startup_account(retained.delivery_id).unwrap(),
        Some(account)
    );
    assert_label_and_binding_liveness(&mut database, &mut *session, &retained);
    diesel::sql_query("UPDATE organization_activation SET admitted=FALSE,billing_hold=TRUE WHERE organization_id=$1")
        .bind::<sql_types::Uuid,_>(record.organization_id).execute(&mut database.connection).unwrap();
    assert_eq!(session.startup_account(retained.delivery_id).unwrap(), None);
    assert_eq!(
        reserve(
            &mut *session,
            &record,
            &EncryptedLaunch(vec![1]),
            &TokenHash::new(vec![5]),
            &budget
        )
        .unwrap(),
        ReservationOutcome::NotPermitted
    );
    diesel::sql_query("UPDATE organization_activation SET admitted=TRUE,billing_hold=FALSE WHERE organization_id=$1")
        .bind::<sql_types::Uuid,_>(record.organization_id).execute(&mut database.connection).unwrap();
    session.disconnect(at()).unwrap();
    assert_eq!(
        reserve(
            &mut *session,
            &record,
            &EncryptedLaunch(vec![1]),
            &TokenHash::new(vec![2]),
            &ReservationBudget {
                current_month_usd: Decimal::new(462, 3),
                next_month_usd: None,
            },
        )
        .unwrap(),
        ReservationOutcome::NotPermitted
    );
}
