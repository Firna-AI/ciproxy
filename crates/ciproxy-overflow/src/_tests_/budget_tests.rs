//! Approved full-lifetime bounds at the public overflow and scaler boundaries.

use chrono::Duration;
use ciproxy_store_interface::overflow::record::{RuntimeCharge, StartPhase, StartRecord};
use rust_decimal::Decimal;
use unimock::Unimock;

use crate::api::Overflow;

use super::fixtures::{Harness, at, capabilities, id, record};

fn ambiguous() -> StartRecord {
    let mut start = record(StartPhase::Creating);
    start.sandbox = None;
    start.size.hourly_usd = Decimal::ONE;
    start.start_quote_usd = Some(Decimal::ZERO);
    start.start_usd = None;
    start
}

fn harness(starts: Vec<StartRecord>) -> Harness {
    let harness = Harness::new(
        capabilities(Unimock::new(())),
        (),
        &["try_lock_overflow", "snapshot", "now", "backend", "open"],
    );
    harness
        .state
        .lock()
        .unwrap()
        .snapshot
        .as_mut()
        .unwrap()
        .starts = starts;
    harness
}

#[tokio::test]
async fn unclear_six_dollar_start_counts_156_then_real_two_dollars_counts_152() {
    let mut confirmed = record(StartPhase::Ended);
    confirmed.delivery_id = id(31);
    confirmed.ends_at = Some(at());
    confirmed.start_usd = Some(Decimal::from(150));
    confirmed.runtime_complete = true;
    let harness = harness(vec![confirmed, ambiguous()]);
    let view = harness.service.view(id(1)).await.unwrap();
    assert_eq!(view.spend.confirmed_usd, Decimal::from(150));
    assert_eq!(view.spend.pending_usd, Decimal::from(6));
    assert_eq!(view.spend.settled_bound_usd, Decimal::ZERO);
    assert_eq!(
        view.scaler_account(at())
            .map(|account| account.spend.spent)
            .map_err(|_| "incomplete"),
        Ok(156_000_000)
    );
    {
        let mut state = harness.state.lock().unwrap();
        let start = &mut state.snapshot.as_mut().unwrap().starts[1];
        start.phase = StartPhase::Ended;
        start.ends_at = Some(at() + Duration::hours(2));
        start.runtime_complete = true;
        start.start_usd = Some(Decimal::ZERO);
        start.runtime = vec![RuntimeCharge {
            month: "2026-11-01".parse().unwrap(),
            usd: Decimal::from(2),
            until: at() + Duration::hours(2),
            running: false,
        }];
    }
    *harness.now.lock().unwrap() = at() + Duration::hours(2);
    harness
        .state
        .lock()
        .unwrap()
        .snapshot
        .as_mut()
        .unwrap()
        .accounts[0]
        .state
        .checked_at = Some(at() + Duration::hours(2));
    let view = harness.service.view(id(1)).await.unwrap();
    assert_eq!(view.spend.confirmed_usd, Decimal::from(152));
    assert_eq!(view.spend.pending_usd, Decimal::ZERO);
    assert_eq!(
        view.scaler_account(at())
            .map(|account| account.spend.spent)
            .map_err(|_| "incomplete"),
        Ok(152_000_000)
    );
}

#[tokio::test]
async fn partial_costs_leave_only_the_remaining_bound_and_expiry_keeps_the_spend() {
    let mut start = ambiguous();
    start.start_quote_usd = Some(Decimal::new(30, 2));
    start.runtime = vec![RuntimeCharge {
        month: "2026-11-01".parse().unwrap(),
        usd: Decimal::from(2),
        until: at() + Duration::hours(2),
        running: true,
    }];
    let harness = harness(vec![start]);
    *harness.now.lock().unwrap() = at() + Duration::hours(2);
    harness
        .state
        .lock()
        .unwrap()
        .snapshot
        .as_mut()
        .unwrap()
        .accounts[0]
        .state
        .checked_at = Some(at() + Duration::hours(2));
    let view = harness.service.view(id(1)).await.unwrap();
    assert_eq!(view.spend.confirmed_usd, Decimal::from(2));
    assert_eq!(view.spend.pending_usd, Decimal::new(43, 1));
    assert_eq!(
        view.scaler_account(at())
            .map(|account| account.spend.spent)
            .map_err(|_| "incomplete"),
        Ok(6_300_000)
    );
    *harness.now.lock().unwrap() = at() + Duration::hours(6);
    harness
        .state
        .lock()
        .unwrap()
        .snapshot
        .as_mut()
        .unwrap()
        .accounts[0]
        .state
        .checked_at = Some(at() + Duration::hours(6));
    let view = harness.service.view(id(1)).await.unwrap();
    assert_eq!(view.spend.confirmed_usd, Decimal::from(2));
    assert_eq!(view.spend.pending_usd, Decimal::ZERO);
    assert_eq!(view.spend.settled_bound_usd, Decimal::new(43, 1));
    assert_eq!(
        view.scaler_account(at())
            .map(|account| account.spend.spent)
            .map_err(|_| "incomplete"),
        Ok(6_300_000)
    );
    assert_eq!(view.starts[0].phase, StartPhase::Creating);
    assert_eq!(view.starts[0].ends_at, None);
    assert_eq!(view.starts[0].sandbox, None);
}

#[tokio::test]
async fn trial_bound_uses_two_hours_and_actual_cost_above_the_bound_still_counts() {
    let mut trial = ambiguous();
    trial.lifetime_seconds = 7200;
    trial.start_quote_usd = Some(Decimal::new(7, 2));
    let harness = harness(vec![trial]);
    let view = harness.service.view(id(1)).await.unwrap();
    assert_eq!(view.spend.confirmed_usd, Decimal::ZERO);
    assert_eq!(view.spend.pending_usd, Decimal::new(207, 2));
    assert_eq!(
        view.scaler_account(at())
            .map(|account| account.spend.spent)
            .map_err(|_| "incomplete"),
        Ok(2_070_000)
    );
    {
        let mut state = harness.state.lock().unwrap();
        let start = &mut state.snapshot.as_mut().unwrap().starts[0];
        start.phase = StartPhase::Ended;
        start.ends_at = Some(at() + Duration::hours(3));
        start.start_usd = Some(Decimal::new(20, 2));
        start.runtime_complete = true;
        start.runtime = vec![RuntimeCharge {
            month: "2026-11-01".parse().unwrap(),
            usd: Decimal::from(3),
            until: at() + Duration::hours(3),
            running: false,
        }];
    }
    *harness.now.lock().unwrap() = at() + Duration::hours(3);
    harness
        .state
        .lock()
        .unwrap()
        .snapshot
        .as_mut()
        .unwrap()
        .accounts[0]
        .state
        .checked_at = Some(at() + Duration::hours(3));
    let view = harness.service.view(id(1)).await.unwrap();
    assert_eq!(view.spend.confirmed_usd, Decimal::new(32, 1));
    assert_eq!(view.spend.pending_usd, Decimal::ZERO);
    assert_eq!(
        view.scaler_account(at())
            .map(|account| account.spend.spent)
            .map_err(|_| "incomplete"),
        Ok(3_200_000)
    );
}

#[tokio::test]
async fn ambiguous_expiry_splits_months_without_repeating_fees_or_ending_authority() {
    let mut start = ambiguous();
    start.starts_at = "2026-11-30T23:00:00Z".parse().unwrap();
    start.start_quote_usd = Some(Decimal::new(30, 2));
    let harness = harness(vec![start]);
    let december = "2026-12-01T00:00:00Z".parse().unwrap();
    *harness.now.lock().unwrap() = december;
    harness
        .state
        .lock()
        .unwrap()
        .snapshot
        .as_mut()
        .unwrap()
        .accounts[0]
        .state
        .checked_at = Some(december);
    let view = harness.service.view(id(1)).await.unwrap();
    assert_eq!(view.spend.confirmed_usd, Decimal::ZERO);
    assert_eq!(view.spend.pending_usd, Decimal::from(5));
    assert_eq!(
        view.scaler_account(december)
            .map(|account| account.spend.spent)
            .map_err(|_| "incomplete"),
        Ok(5_000_000)
    );
    let expires = december + Duration::hours(5);
    *harness.now.lock().unwrap() = expires;
    harness
        .state
        .lock()
        .unwrap()
        .snapshot
        .as_mut()
        .unwrap()
        .accounts[0]
        .state
        .checked_at = Some(expires);
    let view = harness.service.view(id(1)).await.unwrap();
    assert_eq!(view.spend.pending_usd, Decimal::ZERO);
    assert_eq!(view.spend.settled_bound_usd, Decimal::from(5));
    let january = "2027-01-01T00:00:00Z".parse().unwrap();
    *harness.now.lock().unwrap() = january;
    harness
        .state
        .lock()
        .unwrap()
        .snapshot
        .as_mut()
        .unwrap()
        .accounts[0]
        .state
        .checked_at = Some(january);
    let view = harness.service.view(id(1)).await.unwrap();
    assert_eq!(view.spend.pending_usd, Decimal::ZERO);
    assert_eq!(view.spend.settled_bound_usd, Decimal::ZERO);
    assert_eq!(
        view.scaler_account(january)
            .map(|account| account.spend.spent)
            .map_err(|_| "incomplete"),
        Ok(0)
    );
    assert_eq!(view.starts[0].phase, StartPhase::Creating);
    assert_eq!(view.starts[0].ends_at, None);
}
