//! Fallback-only withdrawal stops a durably accepted undersized delivery.

use ciproxy_overflow::api::{Overflow, StartOutcome};
use ciproxy_store_interface::labels::WhenNothingFits;
use ciproxy_store_interface::overflow::record::StartPhase;
use sandbox_interface::SandboxState;

use super::authority_support::{DisableAt, Provider};
use super::fixtures::{id, request};
use super::harness::{Harness, Runtime};

#[tokio::test]
async fn withdrawing_largest_fallback_stops_pending_resume() {
    let provider = Provider::new(DisableAt::Never);
    *provider.inspect_state.lock().unwrap() = SandboxState::Creating;
    let harness = Harness::new(provider.backend, (), Runtime::Start);
    let mut label = harness.database.store.labels(id(1)).unwrap().remove(0);
    label.size.vcpus = 32;
    label.size.memory_gb = 64;
    label.size.disk_gb = 500;
    label.when_nothing_fits = WhenNothingFits::LargestBoat;
    harness.database.store.update_label(id(1), &label).unwrap();
    let mut action = request();
    action.action.requested_size.vcpu = 32;
    action.action.requested_size.memory_bytes = 64_000_000_000;
    action.action.requested_size.disk_bytes = 500_000_000_000;
    assert!(matches!(
        harness.service.start(id(1), action).await.unwrap(),
        StartOutcome::Pending(_)
    ));
    assert_eq!(harness.start(id(11)).phase, StartPhase::Preparing);
    assert_eq!(provider.counts.lock().unwrap().creates, 1);

    label.when_nothing_fits = WhenNothingFits::Wait;
    harness.database.store.update_label(id(1), &label).unwrap();
    *provider.inspect_state.lock().unwrap() = SandboxState::Ready;
    let outcome = harness.service.resume(id(1), id(11)).await.unwrap();
    assert_eq!(provider.counts.lock().unwrap().inputs, 0);
    assert_eq!(provider.counts.lock().unwrap().launches, 0);
    assert_eq!(provider.counts.lock().unwrap().creates, 1);
    assert_eq!(provider.counts.lock().unwrap().recoveries, 0);
    assert_eq!(provider.counts.lock().unwrap().deletes, 1);
    assert_eq!(harness.start(id(11)).phase, StartPhase::Ended);
    assert!(matches!(outcome, StartOutcome::Ended(_)));
}
