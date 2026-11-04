//! Hand-worked corrections for stale sources and observations across midnight.

use std::sync::{Arc, Mutex};

use ciproxy_store_interface::metering::facts::JobPlace;
use ciproxy_store_interface::metering::history::BoatCost;
use ciproxy_store_interface::metering::totals::{AccountedBoat, AccountedJob};
use uuid::Uuid;

use crate::test_support::{at, job, usd};

use super::support::{State, service};

#[test]
fn coherent_boat_snapshot_cannot_restore_a_competing_completed_cost() {
    let open = BoatCost {
        runner_id: Uuid::from_u128(7),
        starts_at: at("2026-11-04T12:00:00Z"),
        ends_at: None,
        start_usd: usd("0"),
        minute_usd: usd("0.02"),
    };
    let state = Arc::new(Mutex::new(State {
        job: None,
        job_cursor: None,
        minutes: usd("0"),
        usd: usd("0.08"),
        cost: open.clone(),
        cost_cursor: AccountedBoat {
            cost: open.clone(),
            observed_at: at("2026-11-04T12:04:00Z"),
        },
        competing_write: Some(AccountedBoat {
            cost: BoatCost {
                ends_at: Some(at("2026-11-04T12:05:00Z")),
                ..open
            },
            observed_at: at("2026-11-04T12:06:00Z"),
        }),
    }));
    let outcome = service(state.clone())
        .refresh(Uuid::from_u128(1), at("2026-11-04T12:10:00Z"))
        .unwrap();
    let state = state.lock().unwrap();
    assert_eq!(state.usd, usd("0.10"));
    assert_eq!((outcome.updated_sandboxes, outcome.stale_sandboxes), (0, 1));
    assert_eq!(
        state.cost_cursor.cost.ends_at,
        Some(at("2026-11-04T12:05:00Z"))
    );
}

#[test]
fn older_refresh_preserves_next_day_cursor_and_corrects_every_affected_date() {
    let open = BoatCost {
        runner_id: Uuid::from_u128(7),
        starts_at: at("2026-11-04T23:59:00Z"),
        ends_at: None,
        start_usd: usd("0"),
        minute_usd: usd("0.02"),
    };
    let previous_job = AccountedJob {
        job: job(1, "2026-11-04T23:59:00Z", None, JobPlace::Machine),
        observed_at: at("2026-11-05T00:01:00Z"),
    };
    let state = Arc::new(Mutex::new(State {
        job: Some(job(
            1,
            "2026-11-04T23:59:00Z",
            Some("2026-11-05T00:00:30Z"),
            JobPlace::Machine,
        )),
        job_cursor: Some(previous_job),
        cost: BoatCost {
            ends_at: Some(at("2026-11-05T00:00:30Z")),
            ..open.clone()
        },
        cost_cursor: AccountedBoat {
            cost: open,
            observed_at: at("2026-11-05T00:01:00Z"),
        },
        minutes: usd("2"),
        usd: usd("0.04"),
        competing_write: None,
    }));
    let service = service(state.clone());
    service
        .refresh(Uuid::from_u128(1), at("2026-11-04T23:59:30Z"))
        .unwrap();
    {
        let state = state.lock().unwrap();
        assert_eq!((state.minutes, state.usd), (usd("1.5"), usd("0.03")));
    }
    let later = service
        .refresh(Uuid::from_u128(1), at("2026-11-05T00:02:00Z"))
        .unwrap();
    let state = state.lock().unwrap();
    assert_eq!((state.minutes, state.usd), (usd("1.5"), usd("0.03")));
    assert_eq!((later.updated_jobs, later.updated_sandboxes), (0, 0));
    assert_eq!(
        state.job_cursor.as_ref().unwrap().observed_at,
        at("2026-11-05T00:01:00Z")
    );
    assert_eq!(state.cost_cursor.observed_at, at("2026-11-05T00:01:00Z"));
}
