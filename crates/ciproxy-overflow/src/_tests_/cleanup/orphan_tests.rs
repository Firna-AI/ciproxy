//! Scoped callback claims and safe orphan cleanup across control-plane outages.

use chrono::Duration;
use ciproxy_store_interface::overflow::known::KnownSandbox;
use ciproxy_store_interface::overflow::record::{RuntimeCharge, StartPhase};
use rust_decimal::Decimal;
use sandbox_interface::{
    BackendManagedSandbox, OperationId, ProviderRef, SandboxConsumer, SandboxId,
    SandboxLifecycleMock, SandboxLifetime, SandboxState,
};
use unimock::{MockFn, Unimock, matching};

use crate::api::{Overflow, Retirement, RunnerAuthorityMock};
use crate::provider::{BoatAccountApiMock, RuntimeUsage};

use super::super::fixtures::{Harness, at, capabilities, id, record};

#[tokio::test]
async fn cleanup_deletes_only_an_unneeded_owned_runner_and_preserves_running_jobs() {
    let backend = capabilities(Unimock::new((
        SandboxLifecycleMock::list_managed_sandboxes
            .some_call(matching!(_))
            .returns(Ok(vec![
                managed(12, 11, "bx_23456789"),
                managed(22, 21, "bx_3456789a"),
                managed(999, 998, "bx_456789ab"),
            ])),
        BoatAccountApiMock::usage
            .some_call(matching!(_, _, _))
            .returns(Ok(None)),
        SandboxLifecycleMock::destroy_sandbox
            .some_call(matching!((id,) if id.as_str() == "bx_3456789a"))
            .returns(Ok(()))
            .once(),
    )));
    let authority = (
        RunnerAuthorityMock::retire
            .some_call(matching!((_, _, runner) if *runner == id(12)))
            .returns(Ok(Retirement::RunningJob)),
        RunnerAuthorityMock::retire
            .some_call(matching!((_, _, runner) if *runner == id(22)))
            .returns(Ok(Retirement::Removed)),
    );
    let harness = Harness::new(
        backend,
        authority,
        &[
            "try_lock_overflow",
            "snapshot",
            "backend",
            "open",
            "save_start",
            "now",
            "collect",
        ],
    );
    let running = record(StartPhase::Running);
    let mut idle = running.clone();
    idle.delivery_id = id(21);
    idle.runner_id = id(22);
    idle.sandbox = Some(KnownSandbox::observed(
        "bx_3456789a".to_owned(),
        idle.starts_at,
        idle.lifetime_seconds,
    ));
    harness
        .state
        .lock()
        .unwrap()
        .snapshot
        .as_mut()
        .unwrap()
        .starts = vec![running.clone(), idle];
    harness.service.cleanup(id(1)).await.unwrap();
    let state = harness.state.lock().unwrap();
    assert_eq!(state.snapshot.as_ref().unwrap().starts[0], running);
    assert_eq!(
        state.snapshot.as_ref().unwrap().starts[1].phase,
        StartPhase::Ended
    );
}

#[tokio::test]
async fn omitted_stopped_sandbox_recovers_its_final_bill_before_lifecycle_end() {
    let backend = capabilities(Unimock::new((
        SandboxLifecycleMock::list_managed_sandboxes
            .some_call(matching!(_))
            .returns(Ok(Vec::new())),
        SandboxLifecycleMock::inspect_sandbox
            .some_call(matching!((sandbox,) if sandbox.as_str() == "bx_23456789"))
            .returns(Ok(sandbox_interface::BackendSandbox {
                provider_ref: ProviderRef::new("bx_23456789"),
                state: SandboxState::Destroyed,
            })),
        BoatAccountApiMock::usage
            .some_call(matching!(_, _, _))
            .answers(&|_, _, since, until| {
                Ok(Some(RuntimeUsage {
                    since: since.unwrap_or(at()),
                    until,
                    usd: Decimal::new(432, 3),
                    running: false,
                }))
            }),
        SandboxLifecycleMock::destroy_sandbox
            .some_call(matching!(_))
            .returns(Ok(())),
    )));
    let harness = Harness::new(
        backend,
        (),
        &[
            "try_lock_overflow",
            "snapshot",
            "backend",
            "open",
            "save_start",
            "now",
            "collect",
        ],
    );
    harness
        .state
        .lock()
        .unwrap()
        .snapshot
        .as_mut()
        .unwrap()
        .starts
        .push(record(StartPhase::Running));
    *harness.now.lock().unwrap() = at() + Duration::hours(7);
    harness.service.cleanup(id(1)).await.unwrap();
    let state = harness.state.lock().unwrap();
    assert_eq!(
        state.snapshot.as_ref().unwrap().starts[0]
            .billing
            .map(|interval| interval.ends_at()),
        Some(at() + Duration::hours(6))
    );
    assert!(state.snapshot.as_ref().unwrap().starts[0].runtime_complete);
    assert_eq!(
        state.snapshot.as_ref().unwrap().starts[0].phase,
        StartPhase::Ended
    );
    assert_eq!(
        state.snapshot.as_ref().unwrap().starts[0].ends_at,
        Some(at() + Duration::hours(7))
    );
}

#[tokio::test]
async fn lost_delete_acknowledgment_never_prices_a_day_of_absent_runtime() {
    let backend = capabilities(Unimock::new((
        SandboxLifecycleMock::list_managed_sandboxes
            .some_call(matching!(_))
            .returns(Ok(Vec::new())),
        BoatAccountApiMock::usage
            .some_call(matching!(_, _, _))
            .answers(&|_, _, _, _| Ok(None)),
        SandboxLifecycleMock::destroy_sandbox
            .some_call(matching!(_))
            .returns(Ok(())),
    )));
    let harness = Harness::new(
        backend,
        (),
        &[
            "try_lock_overflow",
            "snapshot",
            "backend",
            "open",
            "save_start",
            "now",
            "collect",
        ],
    );
    let mut deleting = record(StartPhase::Deleting);
    deleting.runtime = vec![RuntimeCharge {
        month: "2026-11-01".parse().unwrap(),
        usd: Decimal::new(18, 4),
        until: at() + Duration::seconds(90),
        running: true,
    }];
    harness
        .state
        .lock()
        .unwrap()
        .snapshot
        .as_mut()
        .unwrap()
        .starts
        .push(deleting);
    *harness.now.lock().unwrap() = at() + Duration::days(1);
    harness.service.cleanup(id(1)).await.unwrap();
    let mut snapshot = harness.state.lock().unwrap().snapshot.clone().unwrap();
    snapshot.accounts[0].state.checked_at = Some(at() + Duration::days(1));
    harness.state.lock().unwrap().snapshot = Some(snapshot);
    let view = harness.service.view(id(1)).await.unwrap();
    assert_eq!(view.spend.confirmed_usd, Decimal::new(318, 4));
    assert_eq!(view.spend.unmeasured_runtime, [id(11)]);
    assert_eq!(view.spend.pending_usd, Decimal::ZERO);
    assert_eq!(view.spend.settled_bound_usd, Decimal::new(4302, 4));
    assert_eq!(view.scaler_account(at()).unwrap().spend.spent, 462_000);
}

fn managed(runner: u128, delivery: u128, provider: &str) -> BackendManagedSandbox {
    BackendManagedSandbox {
        provider_ref: ProviderRef::new(provider),
        state: SandboxState::Ready,
        sandbox_id: Some(SandboxId::from_uuid(id(runner))),
        operation_id: Some(OperationId::from_uuid(id(delivery))),
        consumer: Some(SandboxConsumer::Runtime),
        lifetime: Some(SandboxLifetime::OneShot {
            max_lifetime: std::time::Duration::from_secs(21600),
        }),
    }
}
