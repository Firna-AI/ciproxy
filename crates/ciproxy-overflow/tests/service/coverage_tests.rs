//! Regressions for missing usage and separate lifecycle and billing facts.

use chrono::Duration;
use ciproxy_overflow::api::{Overflow, StartOutcome};
use ciproxy_overflow::provider::{BoatAccountApiMock, RuntimeUsage};
use ciproxy_store_interface::overflow::billing::MeasuredBillingInterval;
use ciproxy_store_interface::overflow::record::{RuntimeCharge, StartPhase, StartRecord};
use rust_decimal::Decimal;
use sandbox_interface::{ResourceKind, SandboxLifecycleMock};
use unimock::{MockFn, Unimock, matching};

use super::harness::{Harness, Runtime};

use super::fixtures::{at, capabilities, id, record};

pub(super) fn running_observation() -> StartRecord {
    let mut record = record(StartPhase::Running);
    record.runtime = vec![RuntimeCharge {
        month: "2026-11-01".parse().unwrap(),
        usd: Decimal::new(18, 4),
        until: at() + Duration::seconds(90),
        running: true,
    }];
    record.runtime_complete = true;
    record.billing = Some(
        MeasuredBillingInterval::new(
            record.starts_at,
            at() + Duration::seconds(90),
            at() + Duration::seconds(90),
        )
        .0,
    );
    record
}

#[tokio::test]
async fn missing_total_usage_invalidates_saved_running_completeness() {
    missing_usage(true).await;
}

#[tokio::test]
async fn missing_monthly_usage_invalidates_saved_running_completeness() {
    missing_usage(false).await;
}

async fn missing_usage(missing_total: bool) {
    let backend = capabilities(Unimock::new(BoatAccountApiMock::usage.stub(|each| {
        if !missing_total {
            each.call(matching!(_, None, _))
                .answers(&|_, _, _, until| {
                    Ok(Some(RuntimeUsage {
                        since: at(),
                        until,
                        usd: Decimal::new(432, 3),
                        running: false,
                    }))
                })
                .once();
        }
        each.call(matching!(_, _, _))
            .answers(&|_, _, _, _| Ok(None))
            .once();
    })));
    let harness = Harness::new(backend, (), Runtime::Read);
    let now = at() + Duration::hours(7);
    *harness.now.lock().unwrap() = now;
    {
        harness.set_checked(now);
        harness.seed(running_observation());
    }
    let view = harness.service.view(id(1)).await.unwrap();
    assert!(!view.spend.complete(), "missing total: {missing_total}");
    let expected = if missing_total {
        Decimal::new(318, 4)
    } else {
        Decimal::new(462, 3)
    };
    assert_eq!(view.spend.confirmed_usd, expected);
    let saved = harness.snapshot();
    assert!(!saved.starts[0].runtime_complete);
    assert_eq!(saved.starts[0].runtime[0].usd, Decimal::new(18, 4));
    assert_eq!(
        saved.starts[0].billing.map(|interval| interval.ends_at()),
        Some(at() + Duration::seconds(90))
    );
}

#[tokio::test]
async fn absent_resume_recovers_the_final_bill_without_charging_recovery_time() {
    for (runtime_usd, billed_hours) in [(Decimal::new(432, 3), 6), (Decimal::new(576, 3), 7)] {
        let backend = capabilities(Unimock::new((
            SandboxLifecycleMock::inspect_sandbox
                .some_call(matching!(_))
                .returns(Err(sandbox_interface::Error::NotFound {
                    resource: ResourceKind::Sandbox,
                })),
            BoatAccountApiMock::usage
                .some_call(matching!(_, _, _))
                .answers_arc(std::sync::Arc::new(move |_, _, since, until| {
                    Ok(Some(RuntimeUsage {
                        since: since.unwrap_or(at()),
                        until,
                        usd: runtime_usd,
                        running: false,
                    }))
                })),
            SandboxLifecycleMock::destroy_sandbox
                .some_call(matching!(_))
                .returns(Ok(())),
        )));
        let harness = Harness::new(backend, (), Runtime::Read);
        harness.seed(running_observation());
        *harness.now.lock().unwrap() = at() + Duration::hours(7);
        let outcome = harness.service.resume(id(1), id(11)).await.unwrap();
        let StartOutcome::Ended(ended) = outcome else {
            panic!("ended delivery required");
        };
        assert_eq!(ended.ends_at, Some(at() + Duration::hours(7)));
        assert_eq!(
            ended.billing.map(|interval| interval.ends_at()),
            Some(at() + Duration::hours(billed_hours))
        );
        assert!(ended.runtime_complete);
        assert_eq!(ended.runtime[0].usd, runtime_usd);
    }
}

#[tokio::test]
async fn expired_known_sandbox_ends_after_rejected_reads_without_an_invented_bill() {
    let backend = capabilities(Unimock::new((
        SandboxLifecycleMock::list_managed_sandboxes
            .some_call(matching!(_))
            .returns(Err(sandbox_interface::Error::BackendUnavailable {
                backend_id: "boat".to_owned(),
            })),
        SandboxLifecycleMock::inspect_sandbox
            .some_call(matching!(_))
            .returns(Err(sandbox_interface::Error::BackendUnavailable {
                backend_id: "boat".to_owned(),
            })),
        BoatAccountApiMock::usage
            .some_call(matching!(_, _, _))
            .answers(&|_, _, _, _| Err(ciproxy_overflow::error::Error::RejectedKey)),
        SandboxLifecycleMock::destroy_sandbox
            .some_call(matching!(_))
            .returns(Err(sandbox_interface::Error::BackendUnavailable {
                backend_id: "boat".to_owned(),
            })),
    )));
    let harness = Harness::new(backend, (), Runtime::Read);
    harness.seed(running_observation());
    *harness.now.lock().unwrap() = at() + Duration::hours(7);
    harness.service.cleanup(id(1)).await.unwrap();
    let ended = harness.start(id(11));
    assert_eq!(ended.phase, StartPhase::Ended);
    assert_eq!(ended.ends_at, Some(at() + Duration::hours(6)));
    assert!(!ended.runtime_complete);
    assert_eq!(ended.runtime[0].usd, Decimal::new(18, 4));
}

#[tokio::test]
async fn failed_retained_key_cost_read_keeps_replacement_metadata_and_saved_costs() {
    let backend = capabilities(Unimock::new(
        BoatAccountApiMock::usage
            .some_call(matching!(_, _, _))
            .returns(Err(ciproxy_overflow::error::Error::ProviderUnavailable)),
    ));
    let harness = Harness::new(backend, (), Runtime::Read);
    let now = at() + Duration::minutes(5);
    *harness.now.lock().unwrap() = now;
    let mut ended = running_observation();
    ended.phase = StartPhase::Ended;
    ended.ends_at = Some(at() + Duration::minutes(2));
    ended.runtime_complete = false;
    harness.seed(ended);
    let mut replacement = harness.snapshot().accounts[0].clone();
    replacement.account.id = id(44);
    replacement.account.account_name = "Replacement Boat".to_owned();
    replacement.account.connected_at = at() + Duration::minutes(3);
    replacement.state.checked_at = Some(now);
    harness.connect_account(&replacement);
    let view = harness.service.view(id(1)).await.unwrap();
    assert_eq!(view.account.unwrap().id, id(44));
    assert_eq!(view.spend.confirmed_usd, Decimal::new(318, 4));
    assert_eq!(view.spend.unmeasured_runtime, [id(11)]);
}
