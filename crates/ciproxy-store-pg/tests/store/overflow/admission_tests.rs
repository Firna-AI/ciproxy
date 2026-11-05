//! Exact dispatch amounts and the current fallback permission commit atomically.

use ciproxy_store_interface::labels::WhenNothingFits;
use ciproxy_store_interface::overflow::admission::{
    PreparedDispatch, ReservationBudget, ReservationOutcome,
};
use ciproxy_store_interface::overflow::record::{EncryptedLaunch, StartPhase};
use ciproxy_store_interface::secrets::TokenHash;
use rust_decimal::Decimal;

use super::super::support::Database;
use super::support::setup;

#[test]
fn reservation_compares_exact_current_and_next_month_amounts_with_the_saved_limit() {
    let database = Database::new();
    let (workspace_id, _, mut pending) = setup(&database);
    let parameters = pending.dispatch.take().unwrap();
    pending.phase = StartPhase::Unsent;
    let mut settings = database.store.settings(workspace_id).unwrap();
    settings.boat_spend_limit_usd_micros = Some(100_000);
    database
        .store
        .save_settings(workspace_id, &settings)
        .unwrap();
    let mut session = database.store.lock_overflow(workspace_id).unwrap();
    assert!(
        session
            .save_pending(
                &pending,
                &EncryptedLaunch(vec![1]),
                &TokenHash::new(vec![2])
            )
            .unwrap()
    );
    for (current, next, expected) in [
        (
            Decimal::new(1_000_001, 7),
            None,
            ReservationOutcome::SpendLimit,
        ),
        (
            Decimal::new(1, 1),
            Some(Decimal::new(1_000_001, 7)),
            ReservationOutcome::SpendLimit,
        ),
        (
            Decimal::new(1, 1),
            Some(Decimal::new(1, 1)),
            ReservationOutcome::Reserved,
        ),
    ] {
        let dispatch = PreparedDispatch {
            parameters: parameters.clone(),
            starts_at: pending.starts_at,
            budget: ReservationBudget {
                current_month_usd: current,
                next_month_usd: next,
            },
        };
        assert_eq!(
            session.begin_dispatch(&pending, &dispatch).unwrap(),
            expected,
            "current: {current}, next: {next:?}"
        );
    }
    let mut expected = pending;
    expected.phase = StartPhase::Creating;
    expected.dispatch = Some(parameters);
    assert_eq!(session.snapshot().unwrap().starts, [expected]);
}

#[test]
fn reservation_and_dispatch_follow_current_largest_fallback_permission() {
    for dimension in ["cpu", "memory", "disk"] {
        let database = Database::new();
        let (workspace_id, account, mut pending) = setup(&database);
        let mut parameters = pending.dispatch.take().unwrap();
        pending.phase = StartPhase::Unsent;
        match dimension {
            "cpu" => parameters.size.vcpus = 1,
            "memory" => parameters.size.memory_bytes = 7_000_000_000,
            "disk" => parameters.size.disk_bytes = 59_000_000_000,
            _ => unreachable!(),
        }
        let mut label = database.store.labels(workspace_id).unwrap().remove(0);
        let mut session = database.store.lock_overflow(workspace_id).unwrap();
        assert!(
            session
                .save_pending(
                    &pending,
                    &EncryptedLaunch(vec![1]),
                    &TokenHash::new(vec![2])
                )
                .unwrap()
        );
        assert_eq!(
            session.startup_account(pending.delivery_id).unwrap(),
            Some(account)
        );
        let dispatch = PreparedDispatch {
            parameters,
            starts_at: pending.starts_at,
            budget: ReservationBudget {
                current_month_usd: Decimal::new(462, 3),
                next_month_usd: None,
            },
        };
        assert_eq!(
            session.begin_dispatch(&pending, &dispatch).unwrap(),
            ReservationOutcome::NotPermitted
        );
        assert_eq!(session.snapshot().unwrap().starts, [pending.clone()]);
        label.when_nothing_fits = WhenNothingFits::LargestBoat;
        database.store.update_label(workspace_id, &label).unwrap();
        let mut changed_fee = pending.clone();
        changed_fee.start_fee_bound_usd = Some(Decimal::ZERO);
        assert_eq!(
            session.begin_dispatch(&changed_fee, &dispatch).unwrap(),
            ReservationOutcome::NotPermitted
        );
        assert_eq!(session.snapshot().unwrap().starts, [pending.clone()]);
        assert_eq!(
            session.begin_dispatch(&pending, &dispatch).unwrap(),
            ReservationOutcome::Reserved
        );
        label.when_nothing_fits = WhenNothingFits::Wait;
        database.store.update_label(workspace_id, &label).unwrap();
        assert_eq!(session.startup_account(pending.delivery_id).unwrap(), None);
    }
}
