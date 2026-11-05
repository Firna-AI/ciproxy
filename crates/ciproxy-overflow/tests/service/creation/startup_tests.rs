//! New starts and restarts use immutable create and launch identities.

use std::sync::{Arc, Mutex};
use std::time;

use ciproxy_overflow::api::{Overflow, StartOutcome};
use ciproxy_overflow::crypto::SecretPurpose;
use ciproxy_overflow::provider::{BoatAccountApiMock, GuestLauncherMock};
use ciproxy_store_interface::overflow::record::StartPhase;
use sandbox_interface::{
    BackendSandbox, SandboxFilesMock, SandboxLifecycleMock, SandboxLifetime, SandboxState,
};
use unimock::{MockFn, Unimock, matching};

use super::super::harness::{Harness, Runtime};

use super::super::fixtures::{capabilities, facts, id, request};

#[tokio::test]
async fn paid_start_sends_private_input_and_persists_identity_before_launch() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let create_events = events.clone();
    let file_events = events.clone();
    let prepare_events = events.clone();
    let launch_events = events.clone();
    let backend = capabilities(Unimock::new((
        BoatAccountApiMock::account
            .some_call(matching!())
            .answers(&|_| Ok(facts())),
        SandboxLifecycleMock::create_sandbox
            .some_call(matching!(_))
            .answers_arc(Arc::new(move |_, request| {
                assert_eq!(request.operation_id.as_uuid(), id(11));
                assert_eq!(request.sandbox_id.as_uuid(), id(12));
                assert_eq!(request.owner.workspace_id, id(1));
                assert_eq!(request.deployment_id, format!("ciproxy-main/{}", id(1)));
                assert_eq!(
                    request.snapshot_provider_ref.unwrap().as_str(),
                    "shared-image-v7"
                );
                assert_eq!(
                    request.lifetime,
                    SandboxLifetime::OneShot {
                        max_lifetime: time::Duration::from_secs(21600)
                    }
                );
                create_events.lock().unwrap().push("create");
                Ok(BackendSandbox {
                    provider_ref: sandbox_interface::ProviderRef::new("bx_23456789"),
                    state: SandboxState::Ready,
                })
            })),
        SandboxLifecycleMock::inspect_sandbox
            .some_call(matching!(_))
            .returns(Ok(BackendSandbox {
                provider_ref: sandbox_interface::ProviderRef::new("bx_23456789"),
                state: SandboxState::Ready,
            })),
        SandboxFilesMock::write_file
            .some_call(matching!(_))
            .answers_arc(Arc::new(move |_, input| {
                if input.bytes.is_empty() {
                    file_events.lock().unwrap().push("bootstrap");
                    return Ok(());
                }
                assert_eq!(input.path, "registration.json");
                let value: serde_json::Value = serde_json::from_slice(&input.bytes).unwrap();
                assert_eq!(
                    value["configure"]["registration"],
                    "one-use-jit-registration"
                );
                assert_eq!(value["configure"]["cache"], serde_json::Value::Null);
                assert_eq!(value["exit_report_token"], "callback-token-for-one-sandbox");
                assert!(
                    !String::from_utf8(input.bytes)
                        .unwrap()
                        .contains("account-key")
                );
                file_events.lock().unwrap().push("input");
                Ok(())
            })),
        GuestLauncherMock::prepare
            .some_call(matching!(_, _))
            .answers_arc(Arc::new(move |_, _, _| {
                prepare_events.lock().unwrap().push("private");
                Ok(())
            })),
        GuestLauncherMock::launch
            .some_call(matching!(_, _))
            .answers_arc(Arc::new(move |_, _, _| {
                launch_events.lock().unwrap().push("launch");
                Ok(())
            })),
    )));
    let harness = Harness::new(backend, (), Runtime::Start);
    let outcome = harness.service.start(id(1), request()).await.unwrap();
    assert!(matches!(outcome, StartOutcome::Started(_)), "{outcome:?}");
    let state = harness.state.lock().unwrap();
    assert_eq!(
        state.phases,
        [
            StartPhase::Creating,
            StartPhase::Preparing,
            StartPhase::Writing,
            StartPhase::Launching,
            StartPhase::Running
        ]
    );
    assert_eq!(
        *events.lock().unwrap(),
        ["create", "bootstrap", "private", "input", "launch"]
    );
    assert_eq!(state.contexts[0].purpose, SecretPurpose::Launch);
    assert_eq!(state.hashes[&id(11)].as_bytes().len(), 32);
    assert_ne!(state.launches[&id(11)].0, b"one-use-jit-registration");
}
