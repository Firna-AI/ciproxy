//! Exact caller amounts are compared with the saved monthly limit.

use ciproxy_store_interface::labels::WhenNothingFits;
use ciproxy_store_interface::overflow::admission::{ReservationBudget, ReservationOutcome};
use ciproxy_store_interface::overflow::record::{EncryptedLaunch, StartPhase};
use ciproxy_store_interface::secrets::TokenHash;
use rust_decimal::Decimal;

use super::super::support::Database;
use super::support::setup;

#[test]
fn reservation_compares_exact_current_and_next_month_amounts_with_the_saved_limit() {
    let database = Database::new();
    let (workspace_id, _, record) = setup(&database);
    let mut settings = database.store.settings(workspace_id).unwrap();
    settings.boat_spend_limit_usd_micros = Some(100_000);
    database
        .store
        .save_settings(workspace_id, &settings)
        .unwrap();
    let mut session = database.store.lock_overflow(workspace_id).unwrap();
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
        assert_eq!(
            session
                .reserve_start(
                    &record,
                    &EncryptedLaunch(vec![1]),
                    &TokenHash::new(vec![2]),
                    &ReservationBudget {
                        current_month_usd: current,
                        next_month_usd: next,
                    },
                )
                .unwrap(),
            expected,
            "current: {current}, next: {next:?}"
        );
    }
    assert_eq!(session.snapshot().unwrap().starts, [record]);
}

#[test]
fn reservation_and_dispatch_follow_current_largest_fallback_permission() {
    for dimension in ["cpu", "memory", "disk"] {
        let database = Database::new();
        let (workspace_id, account, mut record) = setup(&database);
        record.phase = StartPhase::Unsent;
        match dimension {
            "cpu" => record.size.vcpus = 1,
            "memory" => record.size.memory_bytes = 7_000_000_000,
            "disk" => record.size.disk_bytes = 59_000_000_000,
            _ => unreachable!(),
        }
        let mut label = database.store.labels(workspace_id).unwrap().remove(0);
        let budget = ReservationBudget {
            current_month_usd: Decimal::new(462, 3),
            next_month_usd: None,
        };
        let mut session = database.store.lock_overflow(workspace_id).unwrap();
        assert_eq!(
            session
                .reserve_start(
                    &record,
                    &EncryptedLaunch(vec![1]),
                    &TokenHash::new(vec![2]),
                    &budget
                )
                .unwrap(),
            ReservationOutcome::NotPermitted,
            "{dimension}"
        );
        label.when_nothing_fits = WhenNothingFits::LargestBoat;
        database.store.update_label(workspace_id, &label).unwrap();
        assert_eq!(
            session
                .reserve_start(
                    &record,
                    &EncryptedLaunch(vec![1]),
                    &TokenHash::new(vec![2]),
                    &budget
                )
                .unwrap(),
            ReservationOutcome::Reserved,
            "{dimension}"
        );
        assert_eq!(
            session.startup_account(record.delivery_id).unwrap(),
            Some(account)
        );
        label.when_nothing_fits = WhenNothingFits::Wait;
        database.store.update_label(workspace_id, &label).unwrap();
        assert_eq!(session.startup_account(record.delivery_id).unwrap(), None);
        record.phase = StartPhase::Creating;
        assert_eq!(
            session.begin_dispatch(&record, &budget).unwrap(),
            ReservationOutcome::NotPermitted
        );
    }
}
