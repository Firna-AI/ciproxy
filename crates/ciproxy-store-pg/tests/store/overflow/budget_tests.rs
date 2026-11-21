//! Savings use saved fee bounds without inventing provider-confirmed charges.

use chrono::Duration;
use ciproxy_store_interface::error::Error;
use ciproxy_store_interface::overflow::admission::{ReservationBudget, ReservationOutcome};
use ciproxy_store_interface::overflow::billing::MeasuredBillingInterval;
use ciproxy_store_interface::overflow::known::KnownSandbox;
use ciproxy_store_interface::overflow::record::{EncryptedLaunch, RuntimeCharge, StartPhase};
use ciproxy_store_interface::secrets::TokenHash;
use rust_decimal::Decimal;

use super::super::support::{Database, at};
use super::support::{reserve, setup};

#[test]
fn bounded_start_fee_supplies_savings_without_becoming_a_confirmed_charge() {
    let database = Database::new();
    let (workspace_id, _, mut record) = setup(&database);
    let mut session = database.store.lock_overflow(workspace_id).unwrap();
    record.start_usd = None;
    assert_eq!(
        reserve(
            &mut *session,
            &record,
            &EncryptedLaunch(vec![1]),
            &TokenHash::new(vec![23]),
            &ReservationBudget {
                current_month_usd: Decimal::new(462, 3),
                next_month_usd: None,
            },
        )
        .unwrap(),
        ReservationOutcome::Reserved
    );
    record.sandbox = Some(KnownSandbox::observed(
        "bx_23456789".to_owned(),
        record.starts_at,
        record.dispatch.as_ref().unwrap().lifetime_seconds,
    ));
    record.phase = StartPhase::Ended;
    record.ends_at = Some(at() + Duration::hours(6));
    record.billing = Some(
        MeasuredBillingInterval::new(
            record.starts_at,
            at() + Duration::seconds(90),
            at() + Duration::seconds(90),
        )
        .0,
    );
    record.runtime_complete = true;
    record.runtime = vec![RuntimeCharge {
        month: "2026-11-01".parse().unwrap(),
        usd: Decimal::new(18, 4),
        until: at() + Duration::seconds(90),
        running: false,
    }];
    session.save_start(&record).unwrap();
    let saved = session.snapshot().unwrap();
    assert_eq!(
        saved.starts[0].start_fee_bound_usd,
        Some(Decimal::new(3, 2))
    );
    assert_eq!(saved.starts[0].start_usd, None);
    let bounded = database.store.boat_accounting(workspace_id).unwrap();
    assert_eq!(bounded.costs.len(), 1);
    assert_eq!(bounded.costs[0].start_usd, Decimal::new(3, 2));
    let from = at().date_naive();
    assert_eq!(
        database
            .store
            .savings_snapshot(workspace_id, from, from.succ_opt().unwrap())
            .unwrap()
            .days,
        []
    );
    record.start_usd = Some(Decimal::new(7, 2));
    session.save_start(&record).unwrap();
    let measured = database.store.boat_accounting(workspace_id).unwrap();
    assert_eq!(measured.costs[0].start_usd, Decimal::new(7, 2));
    assert_eq!(
        measured.costs[0].ends_at,
        Some(at() + Duration::seconds(90))
    );
    assert_eq!(
        session.snapshot().unwrap().starts[0].start_fee_bound_usd,
        Some(Decimal::new(3, 2))
    );
    record.start_usd = None;
    record.start_fee_bound_usd = None;
    session.save_start(&record).unwrap();
    assert!(matches!(
        database
            .store
            .savings_snapshot(workspace_id, from, from.succ_opt().unwrap()),
        Err(Error::AccountingIncomplete)
    ));
}
