//! Late running evidence survives a failed monthly observation before settlement.

use chrono::Duration;
use ciproxy_overflow::api::Overflow;
use ciproxy_overflow::error::Error;
use ciproxy_overflow::provider::{BoatAccountApiMock, RuntimeUsage};
use ciproxy_store_interface::overflow::billing::MeasuredBillingInterval;
use ciproxy_store_interface::overflow::record::{RuntimeCharge, RuntimeTotal, StartPhase};
use rust_decimal::Decimal;
use sandbox_interface::SandboxLifecycleMock;
use unimock::{MockFn, Unimock, matching};

use super::harness::{Harness, Runtime};

use super::fixtures::{at, capabilities, id, record};

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
    let harness = Harness::new(backend, (), Runtime::Read);
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
    harness.seed(start);
    for hours in [7, 8] {
        let now = at() + Duration::hours(hours);
        *harness.now.lock().unwrap() = now;
        harness.set_checked(now);
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
    let mut deleting = harness.start(id(11));
    deleting.phase = StartPhase::Deleting;
    harness.save(&deleting);
    assert!(matches!(
        harness.service.cleanup(id(1)).await,
        Err(Error::ProviderUnavailable)
    ));
    assert_eq!(harness.snapshot().starts[0].ends_at, None);
    let now = at() + Duration::hours(14);
    *harness.now.lock().unwrap() = now;
    harness.set_checked(now);
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

    let retained = Harness::new(capabilities(Unimock::new(())), (), Runtime::Read);
    *retained.now.lock().unwrap() = now;
    {
        retained.set_checked(now);
        let mut ended = record(StartPhase::Ended);
        ended.ends_at = Some(now);
        ended.start_usd = None;
        ended.running_observed_at = Some(at() + Duration::hours(7));
        ended.runtime_total = Some(RuntimeTotal {
            since: at(),
            until: at() + Duration::seconds(90),
            usd: Decimal::new(18, 4),
            running: false,
        });
        retained.seed(ended);
        let mut replacement = retained.snapshot().accounts[0].clone();
        retained.database.session().disconnect(now).unwrap();
        assert_eq!(
            retained
                .database
                .store
                .delete_unused_boat_keys(id(1))
                .unwrap(),
            1
        );
        replacement.account.id = id(99);
        replacement.account.connected_at = now;
        retained.connect_account(&replacement);
    }
    let view = retained.service.view(id(1)).await.unwrap();
    assert_eq!(view.spend.confirmed_usd, Decimal::new(18, 4));
    assert_eq!(view.spend.settled_bound_usd, Decimal::new(4602, 4));
    assert_eq!(view.scaler_account(now).unwrap().spend.spent, 462_000);
}
