//! Organization withdrawal stops reserved deliveries without retiring a shared key.

use ciproxy_overflow::api::{Overflow, StartOutcome, WaitReason};
use ciproxy_store_interface::organizations::AppState;
use ciproxy_store_interface::overflow::record::StartPhase;
use sandbox_interface::SandboxState;

use super::authority_actions::{Entry, reconcile};
use super::authority_support::{DisableAt, Provider};
use super::fixtures::{id, request};
use super::harness::{Harness, Runtime};
use super::organization_hooks::{Stage, withdraw};

async fn reserved_withdrawal(state: AppState) {
    for entry in [Entry::Resume, Entry::Duplicate, Entry::Cleanup] {
        let provider = Provider::new(DisableAt::Never);
        *provider.inspect_state.lock().unwrap() = SandboxState::Creating;
        let harness = Harness::new(provider.backend, (), Runtime::Start);
        harness.database.organization(30, 31);
        let pending = harness.service.start(id(1), request()).await.unwrap();
        assert!(matches!(pending, StartOutcome::Pending(_)), "{entry:?}");
        assert_eq!(harness.start(id(11)).phase, StartPhase::Preparing);
        assert_eq!(provider.counts.lock().unwrap().creates, 1);

        withdraw(&harness.database.store, state);
        *provider.inspect_state.lock().unwrap() = SandboxState::Ready;
        reconcile(&harness, entry).await;
        {
            let counts = provider.counts.lock().unwrap();
            assert_eq!(counts.inputs, 0, "{state:?}, {entry:?}");
            assert_eq!(counts.launches, 0, "{state:?}, {entry:?}");
            assert_eq!(counts.bootstraps, 0, "{state:?}, {entry:?}");
            assert_eq!(counts.prepares, 0, "{state:?}, {entry:?}");
            assert_eq!(counts.creates, 1, "{state:?}, {entry:?}");
            assert_eq!(counts.deletes, 1, "{state:?}, {entry:?}");
        }
        assert_eq!(harness.start(id(11)).phase, StartPhase::Ended);
        let account = &harness.snapshot().accounts[0].account;
        assert_eq!(account.retired_at, None);
        assert_eq!(account.rejected_at, None);

        let mut replacement = request();
        replacement.delivery_id = id(34);
        replacement.runner_id = id(35);
        assert_eq!(
            harness.service.start(id(1), replacement).await.unwrap(),
            StartOutcome::Waiting(WaitReason::Disconnected),
            "{state:?}, {entry:?}"
        );
        assert_eq!(harness.snapshot().starts.len(), 1);
        assert_eq!(provider.counts.lock().unwrap().creates, 1);

        let mut other = request();
        other.delivery_id = id(32);
        other.runner_id = id(33);
        other.organization_id = id(30);
        other.scale_set_id = id(31);
        let started = harness.service.start(id(1), other).await.unwrap();
        assert!(matches!(started, StartOutcome::Started(_)), "{entry:?}");
        assert_eq!(harness.start(id(32)).key_id, id(2));
        assert_eq!(provider.counts.lock().unwrap().inputs, 1);
        assert_eq!(provider.counts.lock().unwrap().launches, 1);
        assert_eq!(provider.counts.lock().unwrap().creates, 2);
    }
}

#[tokio::test]
async fn suspended_organization_cannot_resume_a_production_reserved_delivery() {
    reserved_withdrawal(AppState::Suspended).await;
}

#[tokio::test]
async fn removed_organization_cannot_resume_while_another_uses_the_key() {
    reserved_withdrawal(AppState::Removed).await;
}

#[tokio::test]
async fn organization_withdrawal_is_current_at_registration_and_guest_launch() {
    for (stage, state, inputs) in [
        (Stage::Created, AppState::Suspended, 0),
        (Stage::Prepared, AppState::Removed, 0),
        (Stage::InputWritten, AppState::Suspended, 1),
    ] {
        let provider = Provider::new(DisableAt::Never);
        let harness = Harness::new(provider.backend, (), Runtime::Start);
        harness.database.organization(30, 31);
        provider
            .organization
            .set(stage, state, harness.database.store.clone());
        let result = harness.service.start(id(1), request()).await.unwrap();
        let counts = provider.counts.lock().unwrap();
        assert_eq!(counts.inputs, inputs, "{stage:?}");
        assert_eq!(counts.launches, 0, "{stage:?}");
        assert_eq!(counts.creates, 1, "{stage:?}");
        assert_eq!(counts.deletes, 1, "{stage:?}");
        assert!(matches!(result, StartOutcome::Ended(_)), "{stage:?}");
        assert_eq!(harness.start(id(11)).phase, StartPhase::Ended);
    }
}

#[tokio::test]
async fn suspended_after_key_decryption_cannot_dispatch_a_reserved_create() {
    let provider = Provider::new(DisableAt::Never);
    let harness = Harness::new(provider.backend, (), Runtime::Start);
    harness.state.lock().unwrap().suspend_on_key_open = Some((3, harness.database.store.clone()));
    let result = harness.service.start(id(1), request()).await.unwrap();
    let counts = provider.counts.lock().unwrap();
    assert_eq!(counts.creates, 0);
    assert_eq!(counts.inputs, 0);
    assert_eq!(counts.launches, 0);
    assert_eq!(counts.deletes, 0);
    assert_eq!(counts.recoveries, 0);
    assert!(matches!(result, StartOutcome::Ended(_)));
    assert_eq!(harness.start(id(11)).phase, StartPhase::CancelledUnsent);
}
