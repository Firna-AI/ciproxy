//! Measured billing ends before later lifecycle reconciliation.

use chrono::{DateTime, Duration, Utc};
use ciproxy_metering::report::boat::daily_boat_costs;
use ciproxy_metering::report::range::DateRange;
use ciproxy_store_interface::overflow::billing::MeasuredBillingInterval;
use ciproxy_store_interface::overflow::known::KnownSandbox;
use ciproxy_store_interface::overflow::record::{EncryptedLaunch, RuntimeCharge, StartPhase};
use ciproxy_store_interface::secrets::TokenHash;
use rust_decimal::Decimal;

use super::super::support::Database;
use super::support::setup;

#[test]
fn metering_prices_six_billed_hours_before_the_seven_hour_recovery() {
    let database = Database::new();
    let (workspace_id, _, mut record) = setup(&database);
    let start: DateTime<Utc> = "2026-11-05T00:00:00Z".parse().unwrap();
    let mut session = database.store.lock_overflow(workspace_id).unwrap();
    record.starts_at = start;
    assert!(
        session
            .reserve_start(&record, &EncryptedLaunch(vec![1]), &TokenHash::new(vec![3]))
            .unwrap()
    );
    record.sandbox = Some(KnownSandbox::observed(
        "bx_23456789".to_owned(),
        record.starts_at,
        record.lifetime_seconds,
    ));
    record.phase = StartPhase::Running;
    record.runtime_complete = true;
    record.billing = Some(
        MeasuredBillingInterval::new(
            record.starts_at,
            start + Duration::hours(6),
            start + Duration::hours(6),
        )
        .0,
    );
    record.runtime = vec![RuntimeCharge {
        month: "2026-11-01".parse().unwrap(),
        usd: Decimal::new(432, 3),
        until: start + Duration::hours(7),
        running: false,
    }];
    session.save_start(&record).unwrap();
    let range = DateRange {
        from: start.date_naive(),
        until: start.date_naive().succ_opt().unwrap(),
    };
    let read = database.store.boat_accounting(workspace_id).unwrap();
    assert_eq!(read.costs[0].ends_at, Some(start + Duration::hours(6)));
    assert_eq!(
        daily_boat_costs(&read.costs, range, start + Duration::hours(7))[0].usd,
        Decimal::new(462, 3)
    );
    record.phase = StartPhase::Ended;
    record.ends_at = Some(start + Duration::hours(7));
    session.save_start(&record).unwrap();
    let read = database.store.boat_accounting(workspace_id).unwrap();
    assert_eq!(read.costs[0].ends_at, Some(start + Duration::hours(6)));
    assert_eq!(
        daily_boat_costs(&read.costs, range, start + Duration::hours(7))[0].usd,
        Decimal::new(462, 3)
    );
}
