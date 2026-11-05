//! Missing Boat costs do not stop the existing metering retention pass.

use std::sync::Arc;

use chrono::Duration;
use ciproxy_metering::service::{Metering, StoredMetering};
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
fn unknown_boat_interval_preserves_known_costs_and_does_not_block_ninety_day_purge() {
    let database = Database::new();
    let (workspace_id, _, mut record) = setup(&database);
    let mut session = database.store.lock_overflow(workspace_id).unwrap();
    assert_eq!(
        reserve(
            &mut *session,
            &record,
            &EncryptedLaunch(vec![1]),
            &TokenHash::new(vec![4]),
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
    record.phase = StartPhase::Running;
    record.runtime_complete = true;
    record.billing = Some(
        MeasuredBillingInterval::new(
            record.starts_at,
            at() + Duration::seconds(90),
            at() + Duration::seconds(90),
        )
        .0,
    );
    record.runtime = vec![RuntimeCharge {
        month: "2026-11-01".parse().unwrap(),
        usd: Decimal::new(18, 4),
        until: at() + Duration::seconds(90),
        running: true,
    }];
    session.save_start(&record).unwrap();
    let service: Arc<dyn Metering> = Arc::new(StoredMetering::new(database.store.clone()));
    service
        .refresh(workspace_id, at() + Duration::seconds(90))
        .unwrap();
    record.phase = StartPhase::Ended;
    record.ends_at = Some(at() + Duration::hours(7));
    record.runtime_complete = false;
    session.save_start(&record).unwrap();
    let outcome = service.refresh(workspace_id, at() + Duration::days(90));
    assert!(outcome.is_ok(), "{outcome:?}");
    assert_eq!(outcome.unwrap().purged_rows, 1);
    let bounded = database.store.boat_accounting(workspace_id).unwrap();
    assert_eq!(bounded.costs[0].ends_at, Some(at() + Duration::seconds(90)));
    let from = at().date_naive();
    let until = from.succ_opt().unwrap();
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
    record.ends_at = Some(until.and_time(chrono::NaiveTime::MIN).and_utc());
    session.save_start(&record).unwrap();
    assert_eq!(
        database
            .store
            .savings_snapshot(workspace_id, until, until.succ_opt().unwrap())
            .unwrap()
            .days,
        []
    );
}
