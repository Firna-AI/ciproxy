//! Scoped callback claims and safe orphan cleanup across control-plane outages.

use std::sync::{Arc, Mutex};

use chrono::Duration;
use ciproxy_overflow::api::{Overflow, StartOutcome};
use ciproxy_overflow::error::Error;
use ciproxy_overflow::provider::{BoatAccountApiMock, GuestLauncherMock, RuntimeUsage};
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::overflow::admission::{ReservationBudget, ReservationOutcome};
use ciproxy_store_interface::overflow::record::{EncryptedLaunch, StartPhase};
use rust_decimal::Decimal;
use sandbox_interface::{
    BackendSandbox, ProviderRef, SandboxFilesMock, SandboxLifecycleMock, SandboxState,
};
use unimock::{MockFn, Unimock, matching};

use super::super::harness::{Harness, Runtime};

use super::super::fixtures::{at, capabilities, digest, facts, id, record, request, snapshot};

#[tokio::test]
async fn callback_claims_one_runner_once_then_deletes_and_allows_the_next_start() {
    let deletion = Arc::new(Mutex::new(0));
    let count = deletion.clone();
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
        SandboxLifecycleMock::destroy_sandbox
            .some_call(matching!(_))
            .answers_arc(Arc::new(move |_, _| {
                *count.lock().unwrap() += 1;
                Ok(())
            }))
            .once(),
        BoatAccountApiMock::account
            .some_call(matching!())
            .answers(&|_| Ok(facts())),
        SandboxLifecycleMock::create_sandbox
            .some_call(matching!(_))
            .answers(&|_, _| {
                Ok(BackendSandbox {
                    provider_ref: ProviderRef::new("bx_3456789a"),
                    state: SandboxState::Ready,
                })
            }),
        SandboxLifecycleMock::inspect_sandbox
            .some_call(matching!(_))
            .returns(Ok(BackendSandbox {
                provider_ref: ProviderRef::new("bx_3456789a"),
                state: SandboxState::Ready,
            })),
        SandboxFilesMock::write_file
            .some_call(matching!(_))
            .answers(&|_, _| Ok(())),
        GuestLauncherMock::prepare
            .some_call(matching!(_, _))
            .returns(Ok(())),
        GuestLauncherMock::launch
            .some_call(matching!(_, _))
            .returns(Ok(())),
    )));
    let harness = Harness::new(backend, (), Runtime::Start);
    let token = Secret::new("sandbox-only-token");
    harness.seed_token(record(StartPhase::Running), &token);
    harness.set_limit(500_000);
    harness.database.tenant(81);
    {
        let mut other_session = harness.database.store.lock_overflow(id(81)).unwrap();
        let mut account = snapshot().accounts[0].clone();
        account.account.id = id(82);
        let grant = other_session
            .begin_connection(account.account.id)
            .unwrap()
            .unwrap();
        assert!(
            other_session
                .complete_connection(grant, &account, 200_000_000)
                .unwrap()
        );
        let mut other = record(StartPhase::Running);
        other.delivery_id = id(91);
        other.runner_id = id(92);
        other.key_id = id(82);
        other.organization_id = id(83);
        other.label_id = id(84);
        other.scale_set_id = id(85);
        let other_token = digest(&Secret::new("another-workspace-token"));
        assert_eq!(
            other_session
                .reserve_start(
                    &other,
                    &EncryptedLaunch(vec![19]),
                    &other_token,
                    &ReservationBudget {
                        current_month_usd: Decimal::new(462, 3),
                        next_month_usd: None,
                    },
                )
                .unwrap(),
            ReservationOutcome::Reserved
        );
        other_session.save_start(&other).unwrap();
    }
    *harness.now.lock().unwrap() = at() + Duration::seconds(90);
    assert!(matches!(
        harness.service.report_end(token.clone(), id(999)).await,
        Err(Error::InvalidToken)
    ));
    assert!(matches!(
        harness
            .service
            .report_end(Secret::new("another-workspace-token"), id(12))
            .await,
        Err(Error::InvalidToken)
    ));
    assert_eq!(*deletion.lock().unwrap(), 0);
    assert_eq!(
        harness
            .database
            .store
            .lock_overflow(id(81))
            .unwrap()
            .snapshot()
            .unwrap()
            .starts[0]
            .phase,
        StartPhase::Running
    );
    harness
        .service
        .report_end(token.clone(), id(12))
        .await
        .unwrap();
    assert!(matches!(
        harness.service.report_end(token, id(12)).await,
        Err(Error::InvalidToken)
    ));
    assert_eq!(*deletion.lock().unwrap(), 1);
    let view = harness.service.view(id(1)).await.unwrap();
    assert_eq!(view.starts[0].phase, StartPhase::Ended);
    assert_eq!(
        view.starts[0].reported_at,
        Some(at() + Duration::seconds(90))
    );
    assert_eq!(view.spend.confirmed_usd, Decimal::new(318, 4));
    assert!(view.spend.complete());
    let mut next = request();
    next.delivery_id = id(21);
    next.runner_id = id(22);
    assert!(matches!(
        harness.service.start(id(1), next).await.unwrap(),
        StartOutcome::Started(_)
    ));
}
