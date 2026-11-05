//! Retired keys retain inspection and cleanup without startup authority.

use ciproxy_overflow::api::{Overflow, StartOutcome};
use ciproxy_store_interface::overflow::record::StartPhase;

use super::authority_actions::{Entry, Withdrawal, reconcile, withdraw};
use super::authority_support::{DisableAt, Provider};
use super::fixtures::{id, record};
use super::harness::{Harness, Runtime};

async fn revoked_startup(withdrawal: Withdrawal) {
    for phase in [
        StartPhase::Creating,
        StartPhase::Preparing,
        StartPhase::Writing,
        StartPhase::Launching,
    ] {
        for entry in [Entry::Resume, Entry::Duplicate, Entry::Cleanup] {
            let provider = Provider::new(DisableAt::Never);
            let kind = match withdrawal {
                Withdrawal::Replacement => Runtime::Authority,
                _ => Runtime::Read,
            };
            let harness = Harness::new(provider.backend, (), kind);
            let mut pending = record(phase);
            if phase == StartPhase::Creating {
                pending.sandbox = None;
            }
            harness.seed(pending);
            withdraw(&harness, withdrawal).await;
            harness.state.lock().unwrap().active_backends = 0;
            reconcile(&harness, entry).await;
            let counts = provider.counts.lock().unwrap();
            assert_eq!(counts.inputs, 0, "{withdrawal:?}, {entry:?}, {phase:?}");
            assert_eq!(counts.launches, 0, "{withdrawal:?}, {entry:?}, {phase:?}");
            assert_eq!(counts.bootstraps, 0, "{withdrawal:?}, {entry:?}, {phase:?}");
            assert_eq!(counts.prepares, 0, "{withdrawal:?}, {entry:?}, {phase:?}");
            assert_eq!(counts.creates, 0, "{withdrawal:?}, {entry:?}, {phase:?}");
            assert_eq!(counts.deletes, 1, "{withdrawal:?}, {entry:?}, {phase:?}");
            assert_eq!(harness.start(id(11)).phase, StartPhase::Ended);
            assert_eq!(harness.state.lock().unwrap().active_backends, 0);
        }
    }
}

#[tokio::test]
async fn disconnect_cleans_unlaunched_deliveries_on_resume_duplicate_and_cleanup() {
    revoked_startup(Withdrawal::Disconnect).await;
}

#[tokio::test]
async fn replacement_cleans_unlaunched_deliveries_on_resume_duplicate_and_cleanup() {
    revoked_startup(Withdrawal::Replacement).await;
}

#[tokio::test]
async fn disabled_overflow_cleans_unlaunched_deliveries_on_resume_duplicate_and_cleanup() {
    revoked_startup(Withdrawal::Disabled).await;
}

#[tokio::test]
async fn rejected_key_cleans_unlaunched_deliveries_on_resume_duplicate_and_cleanup() {
    revoked_startup(Withdrawal::Rejected).await;
}

async fn disable_during_start(disable_at: DisableAt, inputs: usize) {
    let provider = Provider::new(disable_at);
    let harness = Harness::new(provider.backend, (), Runtime::Read);
    *provider.store.lock().unwrap() = Some(harness.database.store.clone());
    if disable_at == DisableAt::KeyDecrypted {
        harness.state.lock().unwrap().disable_on_key_open =
            Some((2, harness.database.store.clone()));
    }
    harness.seed(record(StartPhase::Preparing));
    let result = harness.service.resume(id(1), id(11)).await.unwrap();
    let counts = provider.counts.lock().unwrap();
    assert_eq!(counts.inputs, inputs);
    assert_eq!(counts.launches, 0);
    assert_eq!(counts.deletes, 1);
    let preparation_count = usize::from(disable_at != DisableAt::KeyDecrypted);
    assert_eq!(counts.bootstraps, preparation_count);
    assert_eq!(counts.prepares, preparation_count);
    assert!(matches!(result, StartOutcome::Ended(_)));
    assert_eq!(harness.start(id(11)).phase, StartPhase::Ended);
}

#[tokio::test]
async fn disabled_before_registration_withdraws_input_and_launch_authority() {
    disable_during_start(DisableAt::Prepared, 0).await;
    disable_during_start(DisableAt::KeyDecrypted, 0).await;
}

#[tokio::test]
async fn disabled_after_registration_withdraws_guest_launch_authority() {
    disable_during_start(DisableAt::InputWritten, 1).await;
}
