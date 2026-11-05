//! Trusted metering facts use provider times and reject unknown final intervals.

use chrono::Duration;
use ciproxy_metering::report::boat::daily_boat_costs;
use ciproxy_metering::report::range::DateRange;
use ciproxy_store_interface::error::Error;
use ciproxy_store_interface::overflow::admission::{ReservationBudget, ReservationOutcome};
use ciproxy_store_interface::overflow::billing::MeasuredBillingInterval;
use ciproxy_store_interface::overflow::known::KnownSandbox;
use ciproxy_store_interface::overflow::record::{EncryptedLaunch, RuntimeCharge, StartPhase};
use ciproxy_store_interface::secrets::TokenHash;
use rust_decimal::Decimal;

use super::super::support::{Database, at};
use super::support::setup;

#[test]
fn metering_uses_provider_start_time_and_never_confirms_an_unknown_outage_day() {
    let database = Database::new();
    let (workspace_id, _, mut record) = setup(&database);
    let mut session = database.store.lock_overflow(workspace_id).unwrap();
    assert_eq!(
        session
            .reserve_start(
                &record,
                &EncryptedLaunch(vec![1]),
                &TokenHash::new(vec![2]),
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
        record.lifetime_seconds,
    ));
    record.phase = StartPhase::Running;
    session.save_start(&record).unwrap();
    assert_eq!(
        database.store.boat_accounting(workspace_id).unwrap().costs,
        []
    );
    record.starts_at = at() + Duration::seconds(30);
    record.runtime_complete = true;
    record.billing = Some(
        MeasuredBillingInterval::new(
            record.starts_at,
            at() + Duration::seconds(120),
            at() + Duration::seconds(120),
        )
        .0,
    );
    record.runtime = vec![RuntimeCharge {
        month: "2026-11-01".parse().unwrap(),
        usd: Decimal::new(18, 4),
        until: at() + Duration::seconds(120),
        running: true,
    }];
    session.save_start(&record).unwrap();
    let live = database.store.boat_accounting(workspace_id).unwrap();
    assert_eq!(live.costs[0].starts_at, at() + Duration::seconds(30));
    assert_eq!(live.costs[0].ends_at, Some(at() + Duration::seconds(120)));
    record.phase = StartPhase::Ended;
    record.ends_at = Some(at() + Duration::seconds(120));
    session.save_start(&record).unwrap();
    let complete = database.store.boat_accounting(workspace_id).unwrap();
    assert_eq!(complete.costs[0].starts_at, at() + Duration::seconds(30));
    assert_eq!(
        complete.costs[0].ends_at,
        Some(at() + Duration::seconds(120))
    );
    let from = at().date_naive();
    let until = from.succ_opt().unwrap();
    let totals = daily_boat_costs(
        &complete.costs,
        DateRange { from, until },
        at() + Duration::days(1),
    );
    assert_eq!(totals[0].usd, Decimal::new(318, 4));
    record.ends_at = Some(at() + Duration::days(1));
    record.runtime_complete = false;
    session.save_start(&record).unwrap();
    let known = database.store.boat_accounting(workspace_id).unwrap();
    assert_eq!(known.costs[0].ends_at, Some(at() + Duration::seconds(120)));
    assert!(matches!(
        database.store.savings_snapshot(workspace_id, from, until),
        Err(Error::AccountingIncomplete)
    ));
    assert_eq!(
        database
            .store
            .savings_snapshot(
                workspace_id,
                from - Duration::days(2),
                from - Duration::days(1)
            )
            .unwrap()
            .days,
        []
    );
}
