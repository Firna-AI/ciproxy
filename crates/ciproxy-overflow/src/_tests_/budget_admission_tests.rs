//! Admission reserves the prospective delivery and requires a safe fee bound.

use chrono::Duration;
use ciproxy_store_interface::overflow::record::StartPhase;
use rust_decimal::Decimal;
use sandbox_interface::SandboxLifecycleMock;
use unimock::{MockFn, Unimock, matching};

use crate::api::{Overflow, StartOutcome, WaitReason};
use crate::error::Error;
use crate::provider::BoatAccountApiMock;

use super::fixtures::{Harness, at, capabilities, facts, id, record, request};

#[tokio::test]
async fn start_reserves_its_own_bound_after_156_dollars_and_waits_when_room_runs_out() {
    let backend = capabilities(Unimock::new((
        BoatAccountApiMock::account
            .some_call(matching!())
            .answers(&|_| Ok(facts())),
        SandboxLifecycleMock::create_sandbox
            .some_call(matching!(_))
            .answers(&|_, _| {
                Err(sandbox_interface::Error::BackendUnavailable {
                    backend_id: "boat".to_owned(),
                })
            }),
    )));
    let harness = Harness::new(
        backend,
        (),
        &[
            "try_lock_overflow",
            "snapshot",
            "now",
            "backend",
            "open",
            "token",
            "seal",
            "save_account",
            "reserve_start",
        ],
    );
    {
        let mut state = harness.state.lock().unwrap();
        let snapshot = state.snapshot.as_mut().unwrap();
        let mut confirmed = record(StartPhase::Ended);
        confirmed.delivery_id = id(30);
        confirmed.start_usd = Some(Decimal::from(150));
        confirmed.ends_at = Some(at());
        confirmed.runtime_complete = true;
        let mut unclear = record(StartPhase::Creating);
        unclear.delivery_id = id(31);
        unclear.sandbox = None;
        unclear.size.hourly_usd = Decimal::ONE;
        unclear.start_quote_usd = Some(Decimal::ZERO);
        unclear.start_usd = None;
        snapshot.starts = vec![confirmed, unclear];
    }
    assert!(matches!(
        harness.service.start(id(1), request()).await.unwrap(),
        StartOutcome::Pending(_)
    ));
    {
        let mut state = harness.state.lock().unwrap();
        let snapshot = state.snapshot.as_mut().unwrap();
        assert_eq!(snapshot.starts[2].start_quote_usd, Some(Decimal::new(3, 2)));
        assert_eq!(snapshot.starts[2].start_usd, None);
        snapshot.spend_limit_usd_micros = Some(156_500_000);
    }
    let mut next = request();
    next.delivery_id = id(41);
    next.runner_id = id(42);
    assert_eq!(
        harness.service.start(id(1), next).await.unwrap(),
        StartOutcome::Waiting(WaitReason::SpendLimit {
            resumes_at: "2026-12-01T00:00:00Z".parse().unwrap(),
        })
    );
    assert_eq!(
        harness
            .state
            .lock()
            .unwrap()
            .snapshot
            .as_ref()
            .unwrap()
            .starts
            .len(),
        3
    );
}

#[tokio::test]
async fn new_start_with_no_quote_never_dispatches_or_saves_an_unbounded_delivery() {
    let backend = capabilities(Unimock::new((
        BoatAccountApiMock::account.stub(|each| {
            each.call(matching!())
                .answers(&|_| {
                    let mut facts = facts();
                    facts.next_start_usd = None;
                    Ok(facts)
                })
                .once()
                .then()
                .answers(&|_| Ok(facts()));
        }),
        SandboxLifecycleMock::create_sandbox
            .some_call(matching!(_))
            .answers(&|_, _| {
                Err(sandbox_interface::Error::BackendUnavailable {
                    backend_id: "boat".to_owned(),
                })
            }),
    )));
    let harness = Harness::new(
        backend,
        (),
        &[
            "try_lock_overflow",
            "snapshot",
            "now",
            "backend",
            "open",
            "token",
            "seal",
            "save_account",
            "reserve_start",
        ],
    );
    assert!(matches!(
        harness.service.start(id(1), request()).await,
        Err(Error::AccountingIncomplete)
    ));
    assert_eq!(
        harness
            .state
            .lock()
            .unwrap()
            .snapshot
            .as_ref()
            .unwrap()
            .starts
            .len(),
        0
    );
    assert!(matches!(
        harness.service.start(id(1), request()).await.unwrap(),
        StartOutcome::Pending(_)
    ));
}

#[tokio::test]
async fn dispatch_near_midnight_checks_the_next_months_existing_bounds() {
    let backend = capabilities(Unimock::new((
        BoatAccountApiMock::account
            .some_call(matching!())
            .answers(&|_| Ok(facts())),
        SandboxLifecycleMock::create_sandbox
            .some_call(matching!(_))
            .answers(&|_, _| {
                Err(sandbox_interface::Error::BackendUnavailable {
                    backend_id: "boat".to_owned(),
                })
            }),
    )));
    let harness = Harness::new(
        backend,
        (),
        &[
            "try_lock_overflow",
            "snapshot",
            "now",
            "backend",
            "open",
            "token",
            "seal",
            "save_account",
            "reserve_start",
        ],
    );
    let now = "2026-11-30T23:00:00Z".parse().unwrap();
    *harness.now.lock().unwrap() = now;
    {
        let mut state = harness.state.lock().unwrap();
        let snapshot = state.snapshot.as_mut().unwrap();
        snapshot.accounts[0].state.checked_at = Some(now);
        snapshot.spend_limit_usd_micros = Some(5_100_000);
        let mut unclear = record(StartPhase::Creating);
        unclear.sandbox = None;
        unclear.starts_at = now;
        unclear.size.hourly_usd = Decimal::ONE;
        unclear.start_quote_usd = Some(Decimal::ZERO);
        unclear.start_usd = None;
        snapshot.starts.push(unclear);
    }
    let mut next = request();
    next.delivery_id = id(41);
    next.runner_id = id(42);
    assert_eq!(
        harness.service.start(id(1), next.clone()).await.unwrap(),
        StartOutcome::Waiting(WaitReason::SpendLimit {
            resumes_at: now + Duration::hours(1),
        })
    );
    assert_eq!(
        harness
            .state
            .lock()
            .unwrap()
            .snapshot
            .as_ref()
            .unwrap()
            .starts
            .len(),
        1
    );
    harness
        .state
        .lock()
        .unwrap()
        .snapshot
        .as_mut()
        .unwrap()
        .spend_limit_usd_micros = Some(6_000_000);
    assert!(matches!(
        harness.service.start(id(1), next).await.unwrap(),
        StartOutcome::Pending(_)
    ));
}
