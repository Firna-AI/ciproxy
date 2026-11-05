//! Runtime plus start fees, exact UTC month boundaries, and immediate limit changes.

use chrono::Duration;
use ciproxy_store_interface::overflow::record::{RuntimeCharge, StartPhase};
use rust_decimal::Decimal;
use sandbox_interface::{PlanSizeRefusalReason, SandboxLifecycleMock, SandboxStartRefusal};
use unimock::{MockFn, Unimock, matching};

use crate::api::{Overflow, StartOutcome, WaitReason};
use crate::error::Error;
use crate::provider::{BoatAccountApiMock, RuntimeUsage};

use super::fixtures::{Harness, at, capabilities, facts, id, record, request};

#[tokio::test]
async fn runtime_and_start_charge_stop_new_starts_and_a_raised_limit_resumes_them() {
    let backend = capabilities(Unimock::new((
        BoatAccountApiMock::usage
            .some_call(matching!(_, _, _))
            .answers(&|_, _, since, until| {
                Ok(Some(RuntimeUsage {
                    since: since.unwrap_or(at()),
                    until,
                    usd: Decimal::new(18, 4),
                    running: true,
                }))
            }),
        BoatAccountApiMock::account
            .some_call(matching!())
            .answers(&|_| Ok(facts())),
        SandboxLifecycleMock::create_sandbox
            .some_call(matching!(_))
            .answers(&|_, _| {
                Err(sandbox_interface::Error::SandboxStartRefused {
                    backend_id: "boat".to_owned(),
                    refusal: SandboxStartRefusal::PlanSize {
                        reason: PlanSizeRefusalReason::PlanRequired,
                    },
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
            "save_start",
            "save_account",
            "reserve_start",
            "save_refusal",
            "token",
            "seal",
        ],
    );
    {
        let mut state = harness.state.lock().unwrap();
        let snapshot = state.snapshot.as_mut().unwrap();
        snapshot.starts.push(record(StartPhase::Running));
        snapshot.spend_limit_usd_micros = Some(31_800);
    }
    *harness.now.lock().unwrap() = at() + Duration::seconds(90);
    let view = harness.service.view(id(1)).await.unwrap();
    assert_eq!(view.spend.confirmed_usd, Decimal::new(318, 4));
    assert!(view.spend.complete());
    let mut next = request();
    next.delivery_id = id(21);
    next.runner_id = id(22);
    assert_eq!(
        harness.service.start(id(1), next.clone()).await.unwrap(),
        StartOutcome::Waiting(WaitReason::SpendLimit {
            resumes_at: "2026-12-01T00:00:00Z".parse().unwrap()
        })
    );
    harness
        .state
        .lock()
        .unwrap()
        .snapshot
        .as_mut()
        .unwrap()
        .spend_limit_usd_micros = Some(1_000_000);
    assert!(matches!(
        harness.service.start(id(1), next).await.unwrap(),
        StartOutcome::Waiting(WaitReason::Refused(_))
    ));
}

#[tokio::test]
async fn new_month_counts_runtime_across_keys_and_organizations_without_repeating_start_fees() {
    let backend = capabilities(Unimock::new(()));
    let harness = Harness::new(
        backend,
        (),
        &["try_lock_overflow", "snapshot", "now", "backend", "open"],
    );
    let december: chrono::DateTime<chrono::Utc> = "2026-12-01T00:01:00Z".parse().unwrap();
    *harness.now.lock().unwrap() = december;
    {
        let mut state = harness.state.lock().unwrap();
        let snapshot = state.snapshot.as_mut().unwrap();
        snapshot.accounts[0].state.checked_at = Some(december);
        let mut previous = record(StartPhase::Ended);
        previous.starts_at = "2026-11-30T23:59:30Z".parse().unwrap();
        previous.ends_at = Some(december);
        previous.runtime_complete = true;
        previous.runtime = vec![
            RuntimeCharge {
                month: "2026-11-01".parse().unwrap(),
                usd: Decimal::new(6, 4),
                until: "2026-12-01T00:00:00Z".parse().unwrap(),
                running: false,
            },
            RuntimeCharge {
                month: "2026-12-01".parse().unwrap(),
                usd: Decimal::new(12, 4),
                until: december,
                running: false,
            },
        ];
        let mut other = record(StartPhase::Ended);
        other.delivery_id = id(31);
        other.runner_id = id(32);
        other.organization_id = id(33);
        other.key_id = id(99);
        other.starts_at = "2026-12-01T00:00:00Z".parse().unwrap();
        other.ends_at = Some(december);
        other.start_usd = Some(Decimal::new(5, 2));
        other.runtime_complete = true;
        other.runtime = vec![RuntimeCharge {
            month: "2026-12-01".parse().unwrap(),
            usd: Decimal::new(12, 4),
            until: december,
            running: false,
        }];
        snapshot.starts = vec![previous, other];
        snapshot.spend_limit_usd_micros = Some(60_000);
    }
    let view = harness.service.view(id(1)).await.unwrap();
    assert_eq!(view.spend.confirmed_usd, Decimal::new(524, 4));
    assert!(view.spend.complete());
    assert_eq!(view.scaler_account(december).unwrap().spend.spent, 52_400);
}

#[tokio::test]
async fn accepted_start_with_no_fee_quote_stays_explicit_in_the_spend_view() {
    let backend = capabilities(Unimock::new(()));
    let harness = Harness::new(
        backend,
        (),
        &["try_lock_overflow", "snapshot", "now", "backend", "open"],
    );
    let mut ended = record(StartPhase::Ended);
    ended.ends_at = Some(at());
    ended.start_usd = None;
    ended.start_quote_usd = None;
    ended.runtime_complete = true;
    harness
        .state
        .lock()
        .unwrap()
        .snapshot
        .as_mut()
        .unwrap()
        .starts
        .push(ended);
    let view = harness.service.view(id(1)).await.unwrap();
    assert_eq!(view.spend.unknown_start_charges, [id(11)]);
    assert!(!view.spend.complete());
    assert!(matches!(
        view.scaler_account(at()),
        Err(Error::AccountingIncomplete)
    ));
    let mut next = request();
    next.delivery_id = id(21);
    next.runner_id = id(22);
    assert!(matches!(
        harness.service.start(id(1), next).await,
        Err(Error::AccountingIncomplete)
    ));
}
