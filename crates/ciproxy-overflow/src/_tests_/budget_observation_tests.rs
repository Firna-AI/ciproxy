//! Late running evidence survives a failed monthly observation before settlement.

use chrono::Duration;
use ciproxy_store_interface::overflow::billing::MeasuredBillingInterval;
use ciproxy_store_interface::overflow::record::{RuntimeCharge, RuntimeTotal, StartPhase};
use rust_decimal::Decimal;
use sandbox_interface::SandboxLifecycleMock;
use unimock::{MockFn, Unimock, matching};

use crate::api::Overflow;
use crate::error::Error;
use crate::provider::{BoatAccountApiMock, RuntimeUsage};

use super::fixtures::{Harness, at, capabilities, id, record};

#[tokio::test]
async fn later_running_evidence_keeps_the_bound_pending_then_final_runtime_replaces_it() {
    let backend = capabilities(Unimock::new((
        BoatAccountApiMock::usage
            .some_call(matching!(_, _, _))
            .answers(&|_, _, since, until| {
                if until == at() + Duration::hours(8) {
                    return Ok(Some(RuntimeUsage {
                        since: since.unwrap_or(at()),
                        until: at() + Duration::seconds(90),
                        usd: Decimal::new(18, 4),
                        running: false,
                    }));
                }
                if since.is_some()
                    && (until == at() + Duration::hours(7) || until == at() + Duration::hours(8))
                {
                    return Ok(None);
                }
                Ok(Some(RuntimeUsage {
                    since: since.unwrap_or(at()),
                    until,
                    usd: Decimal::new(504, 3),
                    running: until == at() + Duration::hours(7),
                }))
            }),
        SandboxLifecycleMock::list_managed_sandboxes
            .some_call(matching!(_))
            .answers(&|_, _| {
                Err(sandbox_interface::Error::BackendUnavailable {
                    backend_id: "boat".to_owned(),
                })
            }),
        SandboxLifecycleMock::destroy_sandbox
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
            "save_start",
            "collect",
        ],
    );
    let mut start = record(StartPhase::Running);
    start.start_usd = None;
    start.runtime_complete = true;
    start.billing = Some(
        MeasuredBillingInterval::new(
            at(),
            at() + Duration::seconds(90),
            at() + Duration::seconds(90),
        )
        .0,
    );
    start.runtime.push(RuntimeCharge {
        month: "2026-11-01".parse().unwrap(),
        usd: Decimal::new(18, 4),
        until: at() + Duration::seconds(90),
        running: false,
    });
    harness
        .state
        .lock()
        .unwrap()
        .snapshot
        .as_mut()
        .unwrap()
        .starts
        .push(start);
    for hours in [7, 8] {
        let now = at() + Duration::hours(hours);
        *harness.now.lock().unwrap() = now;
        harness
            .state
            .lock()
            .unwrap()
            .snapshot
            .as_mut()
            .unwrap()
            .accounts[0]
            .state
            .checked_at = Some(now);
        let view = harness.service.view(id(1)).await.unwrap();
        assert_eq!(view.spend.confirmed_usd, Decimal::new(504, 3));
        assert_eq!(view.spend.pending_usd, Decimal::new(3, 2));
        assert_eq!(view.scaler_account(now).unwrap().spend.spent, 534_000);
        assert_eq!(view.spend.settled_bound_usd, Decimal::ZERO);
        assert_eq!(
            view.starts[0].runtime_total.as_ref().unwrap().until,
            at() + Duration::hours(7)
        );
        assert_eq!(
            view.starts[0].running_observed_at,
            Some(at() + Duration::hours(7))
        );
        assert_eq!(
            view.starts[0].billing.unwrap().ends_at(),
            at() + Duration::seconds(90)
        );
        assert_eq!(view.spend.unmeasured_runtime, [id(11)]);
    }
    harness
        .state
        .lock()
        .unwrap()
        .snapshot
        .as_mut()
        .unwrap()
        .starts[0]
        .phase = StartPhase::Deleting;
    assert!(matches!(
        harness.service.cleanup(id(1)).await,
        Err(Error::ProviderUnavailable)
    ));
    assert_eq!(
        harness
            .state
            .lock()
            .unwrap()
            .snapshot
            .as_ref()
            .unwrap()
            .starts[0]
            .ends_at,
        None
    );
    let now = at() + Duration::hours(14);
    *harness.now.lock().unwrap() = now;
    harness
        .state
        .lock()
        .unwrap()
        .snapshot
        .as_mut()
        .unwrap()
        .accounts[0]
        .state
        .checked_at = Some(now);
    let view = harness.service.view(id(1)).await.unwrap();
    assert_eq!(view.spend.confirmed_usd, Decimal::new(504, 3));
    assert_eq!(view.spend.pending_usd, Decimal::ZERO);
    assert_eq!(view.spend.settled_bound_usd, Decimal::new(3, 2));
    assert_eq!(view.scaler_account(now).unwrap().spend.spent, 534_000);
    assert_eq!(view.starts[0].start_usd, None);
    assert_eq!(
        view.starts[0].billing.unwrap().ends_at(),
        at() + Duration::hours(7)
    );

    let retained = Harness::new(
        capabilities(Unimock::new(())),
        (),
        &[
            "try_lock_overflow",
            "snapshot",
            "now",
            "backend",
            "open",
            "save_start",
        ],
    );
    *retained.now.lock().unwrap() = now;
    {
        let mut state = retained.state.lock().unwrap();
        let snapshot = state.snapshot.as_mut().unwrap();
        snapshot.accounts[0].state.checked_at = Some(now);
        let mut ended = record(StartPhase::Ended);
        ended.key_id = id(99);
        ended.ends_at = Some(now);
        ended.start_usd = None;
        ended.running_observed_at = Some(at() + Duration::hours(7));
        ended.runtime_total = Some(RuntimeTotal {
            since: at(),
            until: at() + Duration::seconds(90),
            usd: Decimal::new(18, 4),
            running: false,
        });
        snapshot.starts.push(ended);
    }
    let view = retained.service.view(id(1)).await.unwrap();
    assert_eq!(view.spend.confirmed_usd, Decimal::new(18, 4));
    assert_eq!(view.spend.settled_bound_usd, Decimal::new(4602, 4));
    assert_eq!(view.scaler_account(now).unwrap().spend.spent, 462_000);
}
