//! Confirmed Free-to-paid period persistence through the public store API.

use ciproxy_store_interface::metering::history::Period;

use super::super::support::Database;
use super::support::time;

#[test]
fn confirmed_paid_period_closes_free_and_reads_ordered_workspace_boundaries() {
    let database = Database::new();
    let workspace = database.legacy_workspace();
    let other = database.legacy_workspace();
    let store = &database.store;
    let free = Period {
        starts_at: time("2026-11-01T00:00:00Z"),
        ends_at: time("2026-12-01T00:00:00Z"),
    };
    let paid = Period {
        starts_at: time("2026-11-04T15:00:00Z"),
        ends_at: time("2026-12-04T15:00:00Z"),
    };
    store.save_metering_period(workspace.id, &free).unwrap();
    store.save_metering_period(other.id, &free).unwrap();
    store.save_metering_period(workspace.id, &paid).unwrap();
    store
        .save_metering_period(
            workspace.id,
            &Period {
                ends_at: time("2026-11-04T15:00:00Z"),
                ..free
            },
        )
        .unwrap();
    assert_eq!(
        store.metering_periods(workspace.id).unwrap(),
        vec![
            Period {
                starts_at: time("2026-11-01T00:00:00Z"),
                ends_at: time("2026-11-04T15:00:00Z")
            },
            Period {
                starts_at: time("2026-11-04T15:00:00Z"),
                ends_at: time("2026-12-04T15:00:00Z")
            },
        ]
    );
    assert_eq!(
        store.metering_periods(other.id).unwrap(),
        vec![Period {
            starts_at: time("2026-11-01T00:00:00Z"),
            ends_at: time("2026-12-01T00:00:00Z")
        },]
    );
}
