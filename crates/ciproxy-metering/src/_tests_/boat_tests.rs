//! Idle-sandbox start fees and incremental elapsed runtime costs.

use ciproxy_store_interface::metering::history::BoatCost;
use ciproxy_store_interface::metering::totals::AccountedBoat;
use uuid::Uuid;

use crate::report::boat::{boat_cost_deltas, daily_boat_costs};
use crate::test_support::{at, range, usd};

#[test]
fn idle_boat_cost_includes_start_fee_and_runtime_across_midnight() {
    let cost = BoatCost {
        runner_id: Uuid::from_u128(7),
        starts_at: at("2026-11-04T23:59:00Z"),
        ends_at: Some(at("2026-11-05T00:01:00Z")),
        start_usd: usd("0.10"),
        minute_usd: usd("0.02"),
    };
    let days = daily_boat_costs(
        std::slice::from_ref(&cost),
        range("2026-11-04", "2026-11-06"),
        at("2026-11-05T00:02:00Z"),
    );
    assert_eq!((days[0].usd, days[1].usd), (usd("0.12"), usd("0.02")));
    let before = AccountedBoat {
        cost: BoatCost {
            ends_at: None,
            ..cost.clone()
        },
        observed_at: at("2026-11-05T00:00:00Z"),
    };
    let after = AccountedBoat {
        cost,
        observed_at: at("2026-11-05T00:02:00Z"),
    };
    let changes = boat_cost_deltas(Some(&before), &after, range("2026-11-04", "2026-11-06"));
    assert_eq!((changes.len(), changes[0].usd), (1, usd("0.02")));
    assert_eq!(
        boat_cost_deltas(Some(&after), &after, range("2026-11-04", "2026-11-06")),
        vec![]
    );
}
