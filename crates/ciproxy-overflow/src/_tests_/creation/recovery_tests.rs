//! New starts and restarts use immutable create and launch identities.

use std::time;

use chrono::Duration;
use ciproxy_store_interface::overflow::record::StartPhase;
use sandbox_interface::{
    BackendSandbox, SandboxFilesMock, SandboxLifecycleMock, SandboxLifetime, SandboxState,
};
use unimock::{MockFn, Unimock, matching};

use crate::api::{Overflow, StartOutcome};
use crate::error::Error;
use crate::provider::{BoatAccountApiMock, GuestLauncherMock};

use super::super::fixtures::{Harness, at, capabilities, facts, id, record, request};

#[tokio::test]
async fn ambiguous_create_uses_recovery_after_the_provider_idempotency_window() {
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
            })
            .once(),
        SandboxLifecycleMock::recover_sandbox_create
            .some_call(matching!(_))
            .answers(&|_, request| {
                assert_eq!(request.operation_id.as_uuid(), id(11));
                assert_eq!(
                    request.lifetime,
                    SandboxLifetime::OneShot {
                        max_lifetime: time::Duration::from_secs(21600)
                    }
                );
                Ok(None)
            })
            .once(),
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
        harness.service.start(id(1), request()).await.unwrap(),
        StartOutcome::Pending(_)
    ));
    *harness.now.lock().unwrap() = at() + Duration::hours(25);
    assert!(matches!(
        harness.service.resume(id(1), id(11)).await.unwrap(),
        StartOutcome::Pending(_)
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
        1
    );
    let state = harness.state.lock().unwrap();
    let pending = &state.snapshot.as_ref().unwrap().starts[0];
    assert_eq!(pending.sandbox, None);
    assert_eq!(pending.ends_at, None);
}

#[tokio::test]
async fn launch_recovery_keeps_the_consumed_registration_and_same_delivery() {
    let backend = capabilities(Unimock::new((
        SandboxLifecycleMock::inspect_sandbox
            .some_call(matching!(_))
            .returns(Ok(BackendSandbox {
                provider_ref: sandbox_interface::ProviderRef::new("bx_23456789"),
                state: SandboxState::Ready,
            })),
        GuestLauncherMock::launch
            .some_call(matching!((_, delivery) if *delivery == id(11)))
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
        .push(record(StartPhase::Launching));
    let outcome = harness.service.resume(id(1), id(11)).await.unwrap();
    assert!(matches!(outcome, StartOutcome::Started(_)), "{outcome:?}");
    assert_eq!(harness.state.lock().unwrap().phases, [StartPhase::Running]);
}

#[tokio::test]
async fn uncertain_file_write_destroys_the_sandbox_before_any_replacement() {
    let backend = capabilities(Unimock::new((
        SandboxLifecycleMock::inspect_sandbox
            .some_call(matching!(_))
            .returns(Ok(BackendSandbox {
                provider_ref: sandbox_interface::ProviderRef::new("bx_23456789"),
                state: SandboxState::Ready,
            })),
        SandboxFilesMock::write_file
            .some_call(matching!(_))
            .returns(Err(sandbox_interface::Error::FileWriteUnconfirmed))
            .once(),
        BoatAccountApiMock::usage
            .some_call(matching!(_, _, _))
            .returns(Ok(None)),
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
            "encrypted_launch",
            "save_start",
            "now",
        ],
    );
    {
        let mut state = harness.state.lock().unwrap();
        state
            .snapshot
            .as_mut()
            .unwrap()
            .starts
            .push(record(StartPhase::Preparing));
        state.launches.insert(
            id(11),
            ciproxy_store_interface::overflow::record::EncryptedLaunch(vec![1]),
        );
        state
            .sealed
            .insert((id(11), 2), crate::secrets::SecretBytes::new(vec![9]));
    }
    assert!(matches!(
        harness.service.resume(id(1), id(11)).await,
        Err(Error::ProviderUnavailable)
    ));
    assert_eq!(
        harness.state.lock().unwrap().phases,
        [
            StartPhase::Writing,
            StartPhase::Deleting,
            StartPhase::Deleting,
            StartPhase::Ended
        ]
    );
    assert!(matches!(
        harness.service.resume(id(1), id(11)).await.unwrap(),
        StartOutcome::Ended(_)
    ));
}

#[tokio::test]
async fn write_crash_recovery_deletes_without_replaying_the_file_transfer() {
    let backend = capabilities(Unimock::new((
        BoatAccountApiMock::usage
            .some_call(matching!(_, _, _))
            .returns(Ok(None)),
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
        .push(record(StartPhase::Writing));
    let outcome = harness.service.resume(id(1), id(11)).await.unwrap();
    assert!(matches!(outcome, StartOutcome::Ended(_)), "{outcome:?}");
    assert_eq!(
        harness.state.lock().unwrap().phases,
        [
            StartPhase::Deleting,
            StartPhase::Deleting,
            StartPhase::Ended
        ]
    );
}
