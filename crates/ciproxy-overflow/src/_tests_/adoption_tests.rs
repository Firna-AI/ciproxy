//! Cleanup persists recovered identity and lifetime proof before demand checks.

use std::sync::{Arc, Mutex};

use chrono::Duration;
use ciproxy_store_interface::overflow::record::{EncryptedLaunch, StartPhase};
use sandbox_interface::{
    BackendManagedSandbox, BackendSandbox, ProviderRef, SandboxLifecycleMock, SandboxState,
};
use unimock::{MockFn, Unimock, matching};

use crate::api::{Overflow, Retirement, RunnerAuthorityMock};
use crate::error::Error;
use crate::provider::BoatAccountApiMock;

use super::fixtures::{Harness, State, at, capabilities, id, record};

#[tokio::test]
async fn cleanup_saves_recovered_identity_and_lifetime_before_waiting_demand_returns() {
    let backend = capabilities(Unimock::new((
        SandboxLifecycleMock::list_managed_sandboxes
            .some_call(matching!(_))
            .returns(Ok(vec![listed(SandboxState::Ready)])),
        SandboxLifecycleMock::recover_sandbox_create
            .some_call(matching!(_))
            .returns(Ok(Some(BackendSandbox {
                provider_ref: ProviderRef::new("bx_23456789"),
                state: SandboxState::Ready,
            }))),
    )));
    let state = Arc::new(Mutex::new(None::<Arc<Mutex<State>>>));
    let observed = state.clone();
    let authority = RunnerAuthorityMock::retire
        .some_call(matching!(_, _, _))
        .answers_arc(Arc::new(move |_, _, _, _| {
            let captured = observed.lock().unwrap().clone().unwrap();
            let captured = captured.lock().unwrap();
            let pending = &captured.snapshot.as_ref().unwrap().starts[0];
            assert_eq!(pending.phase, StartPhase::Preparing);
            assert_eq!(
                pending.sandbox.as_ref().map(|sandbox| sandbox.id()),
                Some("bx_23456789")
            );
            assert_eq!(
                pending
                    .sandbox
                    .as_ref()
                    .map(|sandbox| sandbox.lifetime_ends_by()),
                Some(at() + Duration::hours(6))
            );
            Ok(Retirement::WaitingDemand)
        }));
    let harness = Harness::new(
        backend,
        authority,
        &[
            "try_lock_overflow",
            "snapshot",
            "save_start",
            "now",
            "backend",
            "open",
            "collect",
        ],
    );
    *state.lock().unwrap() = Some(harness.state.clone());
    seed_creating(&harness);
    harness.service.cleanup(id(1)).await.unwrap();
    let state = harness.state.lock().unwrap();
    let saved = &state.snapshot.as_ref().unwrap().starts[0];
    assert_eq!(
        saved.sandbox.as_ref().map(|sandbox| sandbox.id()),
        Some("bx_23456789")
    );
    assert_eq!(
        saved
            .sandbox
            .as_ref()
            .map(|sandbox| sandbox.lifetime_ends_by()),
        Some(at() + Duration::hours(6))
    );
    assert_eq!(saved.phase, StartPhase::Preparing);
    assert_eq!(state.phases, [StartPhase::Preparing]);
}

#[tokio::test]
async fn failed_recovered_cleanup_keeps_the_original_bound_after_old_key_rejection() {
    let attempts = Arc::new(Mutex::new(0));
    let reads = attempts.clone();
    let backend = capabilities(Unimock::new((
        SandboxLifecycleMock::list_managed_sandboxes
            .some_call(matching!(_))
            .answers_arc(Arc::new(move |_, _| {
                let mut count = reads.lock().unwrap();
                *count += 1;
                if *count == 1 {
                    Ok(vec![listed(SandboxState::Ready)])
                } else {
                    Err(sandbox_interface::Error::BackendUnavailable {
                        backend_id: "boat".to_owned(),
                    })
                }
            })),
        SandboxLifecycleMock::recover_sandbox_create
            .some_call(matching!(_))
            .returns(Ok(Some(BackendSandbox {
                provider_ref: ProviderRef::new("bx_23456789"),
                state: SandboxState::Ready,
            }))),
        BoatAccountApiMock::usage
            .some_call(matching!(_, _, _))
            .answers(&|_, _, _, _| Err(Error::RejectedKey)),
        SandboxLifecycleMock::destroy_sandbox
            .some_call(matching!(_))
            .answers(&|_, _| {
                Err(sandbox_interface::Error::BackendUnavailable {
                    backend_id: "boat".to_owned(),
                })
            }),
    )));
    let authority = RunnerAuthorityMock::retire
        .some_call(matching!(_, _, _))
        .returns(Ok(Retirement::Removed));
    let harness = Harness::new(
        backend,
        authority,
        &[
            "try_lock_overflow",
            "snapshot",
            "save_start",
            "now",
            "backend",
            "open",
            "collect",
        ],
    );
    seed_creating(&harness);
    assert!(matches!(
        harness.service.cleanup(id(1)).await,
        Err(Error::ProviderUnavailable)
    ));
    {
        let mut state = harness.state.lock().unwrap();
        let snapshot = state.snapshot.as_mut().unwrap();
        snapshot.accounts[0].account.retired_at = Some(at() + Duration::hours(1));
        snapshot.accounts[0].account.rejected_at = Some(at() + Duration::hours(1));
        let mut replacement = snapshot.accounts[0].clone();
        replacement.account.id = id(44);
        replacement.account.connected_at = at() + Duration::hours(1);
        replacement.account.retired_at = None;
        replacement.account.rejected_at = None;
        snapshot.accounts.push(replacement);
    }
    *harness.now.lock().unwrap() = at() + Duration::hours(7);
    let outcome = harness.service.cleanup(id(1)).await;
    assert!(outcome.is_ok(), "{outcome:?}");
    let state = harness.state.lock().unwrap();
    let ended = &state.snapshot.as_ref().unwrap().starts[0];
    assert_eq!(ended.ends_at, Some(at() + Duration::hours(6)));
    assert_eq!(ended.phase, StartPhase::Ended);
    assert!(!ended.runtime_complete);
}

#[tokio::test]
async fn recovered_destroyed_sandbox_ends_despite_waiting_demand() {
    end_destroyed(true).await;
}

#[tokio::test]
async fn listed_destroyed_sandbox_ends_despite_waiting_demand() {
    end_destroyed(false).await;
}

async fn end_destroyed(recovering: bool) {
    let common = (
        SandboxLifecycleMock::list_managed_sandboxes
            .some_call(matching!(_))
            .returns(Ok(vec![listed(SandboxState::Destroyed)])),
        BoatAccountApiMock::usage
            .some_call(matching!(_, _, _))
            .returns(Ok(None)),
        SandboxLifecycleMock::destroy_sandbox
            .some_call(matching!(_))
            .returns(Ok(())),
    );
    let backend = if recovering {
        capabilities(Unimock::new((
            common,
            SandboxLifecycleMock::recover_sandbox_create
                .some_call(matching!(_))
                .returns(Ok(Some(BackendSandbox {
                    provider_ref: ProviderRef::new("bx_23456789"),
                    state: SandboxState::Destroyed,
                }))),
        )))
    } else {
        capabilities(Unimock::new(common))
    };
    let harness = Harness::new(
        backend,
        (),
        &[
            "try_lock_overflow",
            "snapshot",
            "save_start",
            "now",
            "backend",
            "open",
            "collect",
        ],
    );
    if recovering {
        seed_creating(&harness);
    } else {
        harness
            .state
            .lock()
            .unwrap()
            .snapshot
            .as_mut()
            .unwrap()
            .starts
            .push(record(StartPhase::Running));
    }
    harness.service.cleanup(id(1)).await.unwrap();
    assert_eq!(
        harness
            .state
            .lock()
            .unwrap()
            .snapshot
            .as_ref()
            .unwrap()
            .starts[0]
            .phase,
        StartPhase::Ended
    );
}

fn listed(state: SandboxState) -> BackendManagedSandbox {
    BackendManagedSandbox {
        provider_ref: ProviderRef::new("bx_23456789"),
        state,
        sandbox_id: None,
        operation_id: None,
        consumer: None,
        lifetime: None,
    }
}

fn seed_creating(harness: &Harness) {
    let mut pending = record(StartPhase::Creating);
    pending.sandbox = None;
    let mut state = harness.state.lock().unwrap();
    state.snapshot.as_mut().unwrap().starts.push(pending);
    state.launches.insert(id(11), EncryptedLaunch(vec![17]));
}
