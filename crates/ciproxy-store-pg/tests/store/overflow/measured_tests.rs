//! Live measurements never authorize costs beyond their observed prefix.

use chrono::Duration;
use ciproxy_metering::service::{Metering, StoredMetering};
use ciproxy_store_interface::overflow::billing::MeasuredBillingInterval;
use ciproxy_store_interface::overflow::known::KnownSandbox;
use ciproxy_store_interface::overflow::record::{EncryptedLaunch, RuntimeCharge, StartPhase};
use ciproxy_store_interface::secrets::TokenHash;
use rust_decimal::Decimal;

use super::super::support::{Database, at};
use super::support::setup;

#[test]
fn late_metering_refresh_prices_only_the_ninety_second_measured_prefix() {
    let database = Database::new();
    let (workspace_id, _, mut record) = setup(&database);
    let mut session = database.store.lock_overflow(workspace_id).unwrap();
    assert!(
        session
            .reserve_start(
                &record,
                &EncryptedLaunch(vec![1]),
                &TokenHash::new(vec![17])
            )
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
            at() + Duration::hours(8),
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
    let service: &dyn Metering = &StoredMetering::new(database.store.clone());
    service
        .refresh(workspace_id, at() + Duration::hours(7))
        .unwrap();
    let day = at().date_naive();
    let snapshot = database
        .store
        .savings_snapshot(workspace_id, day, day.succ_opt().unwrap())
        .unwrap();
    assert_eq!(snapshot.days[0].boat_usd, Decimal::new(318, 4));
    assert_eq!(
        database.store.boat_accounting(workspace_id).unwrap().costs[0].ends_at,
        Some(at() + Duration::seconds(90))
    );
}
