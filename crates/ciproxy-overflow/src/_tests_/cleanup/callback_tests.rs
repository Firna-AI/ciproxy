//! Scoped callback claims and safe orphan cleanup across control-plane outages.

use std::sync::{Arc, Mutex};

use chrono::Duration;
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::overflow::record::StartPhase;
use rust_decimal::Decimal;
use sandbox_interface::{
    BackendSandbox, ProviderRef, SandboxFilesMock, SandboxLifecycleMock, SandboxState,
};
use unimock::{MockFn, Unimock, matching};

use crate::api::{Overflow, StartOutcome};
use crate::error::Error;
use crate::provider::{BoatAccountApiMock, GuestLauncherMock, RuntimeUsage};
use crate::secrets::digest;

use super::super::fixtures::{Harness, at, capabilities, facts, id, record, request};

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
    let harness = Harness::new(
        backend,
        (),
        &[
            "lock_overflow",
            "snapshot",
            "now",
            "backend",
            "open",
            "overflow_token_workspace",
            "accept_end",
            "save_start",
            "collect",
            "try_lock_overflow",
            "token",
            "seal",
            "save_account",
            "reserve_start",
            "encrypted_launch",
        ],
    );
    let token = Secret::new("sandbox-only-token");
    {
        let mut state = harness.state.lock().unwrap();
        state
            .snapshot
            .as_mut()
            .unwrap()
            .starts
            .push(record(StartPhase::Running));
        state.snapshot.as_mut().unwrap().spend_limit_usd_micros = Some(50_000);
        state.hashes.insert(id(11), digest(&token));
        let mut accepted = record(StartPhase::Deleting);
        accepted.reported_at = Some(at() + Duration::seconds(90));
        state.end_claims.extend([None, Some(accepted), None]);
        state
            .token_lookups
            .extend([Some(id(1)), None, Some(id(1)), Some(id(1))]);
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
    assert_eq!(view.spend.known_usd, Decimal::new(318, 4));
    assert!(view.spend.complete());
    let mut next = request();
    next.delivery_id = id(21);
    next.runner_id = id(22);
    assert!(matches!(
        harness.service.start(id(1), next).await.unwrap(),
        StartOutcome::Started(_)
    ));
}
