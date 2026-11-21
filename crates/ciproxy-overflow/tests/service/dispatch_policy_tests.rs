//! Current label policy and saved own-fit context control the largest-size exception.

use chrono::Duration;
use ciproxy_overflow::api::{Overflow, StartOutcome, WaitReason};
use ciproxy_overflow::error::Error;
use ciproxy_scaler::actions::{Destination, StartReason};
use ciproxy_scaler::boat::BoatSizeId;
use ciproxy_store_interface::labels::WhenNothingFits;
use ciproxy_store_interface::overflow::account::SizeOffer;
use rust_decimal::Decimal;

use super::dispatch_support::Provider;
use super::fixtures::{at, id, request};
use super::harness::{Harness, Runtime};

#[tokio::test]
async fn unsent_selection_obeys_current_wait_and_largest_choice() {
    let provider = Provider::new(false);
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
    harness.state.lock().unwrap().fail_on_key_open = Some(2);
    assert!(matches!(
        harness.service.start(id(1), action).await,
        Err(Error::KmsHttp { status: 503 })
    ));
    let pending = harness.start(id(11));
    label.when_nothing_fits = WhenNothingFits::Wait;
    harness.database.store.update_label(id(1), &label).unwrap();
    assert_eq!(
        harness.service.resume(id(1), id(11)).await.unwrap(),
        StartOutcome::Waiting(WaitReason::SizeUnavailable)
    );
    assert_eq!(harness.start(id(11)), pending);
    label.when_nothing_fits = WhenNothingFits::LargestBoat;
    harness.database.store.update_label(id(1), &label).unwrap();
    let store = harness.database.store.clone();
    let mut current = label.clone();
    harness.state.lock().unwrap().dispatch_effect = Some(Box::new(move || {
        current.when_nothing_fits = WhenNothingFits::Wait;
        store.update_label(id(1), &current).unwrap();
    }));
    assert_eq!(
        harness.service.resume(id(1), id(11)).await.unwrap(),
        StartOutcome::Waiting(WaitReason::SizeUnavailable)
    );
    assert_eq!(harness.start(id(11)), pending);
    assert_eq!(provider.creates.lock().unwrap().len(), 0);
    harness.database.store.update_label(id(1), &label).unwrap();
    assert!(matches!(
        harness.service.resume(id(1), id(11)).await.unwrap(),
        StartOutcome::Pending(_)
    ));
    assert_eq!(provider.creates.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn catalog_downgrade_preserves_scaler_own_machine_fit_context() {
    let provider = Provider::new(false);
    let harness = Harness::new(provider.backend, (), Runtime::Start);
    let mut label = harness.database.store.labels(id(1)).unwrap().remove(0);
    label.size.vcpus = 16;
    label.size.memory_gb = 32;
    label.size.disk_gb = 180;
    label.when_nothing_fits = WhenNothingFits::LargestBoat;
    harness.database.store.update_label(id(1), &label).unwrap();
    let mut account = harness.snapshot().accounts.remove(0);
    account.state.sizes.push(SizeOffer {
        name: "xlarge".to_owned(),
        vcpus: 16,
        memory_bytes: 32_000_000_000,
        disk_bytes: 251_000_000_000,
        hourly_usd: Decimal::new(2, 1),
    });
    harness.save_account(&account);
    let mut action = request();
    action.action.requested_size.vcpu = 16;
    action.action.requested_size.memory_bytes = 32_000_000_000;
    action.action.requested_size.disk_bytes = 180_000_000_000;
    action.action.size.vcpu = 16;
    action.action.size.memory_bytes = 32_000_000_000;
    action.action.size.disk_bytes = 251_000_000_000;
    action.action.destination = Destination::Boat(BoatSizeId("xlarge".to_owned()));
    action.action.reason = StartReason::OverflowWaitElapsed {
        waited: std::time::Duration::from_secs(60),
    };
    harness.state.lock().unwrap().fail_on_key_open = Some(2);
    assert!(matches!(
        harness.service.start(id(1), action).await,
        Err(Error::KmsHttp { status: 503 })
    ));
    let pending = harness.start(id(11));
    assert!(pending.own_machine_fits);
    *harness.now.lock().unwrap() = at() + Duration::minutes(5);
    assert_eq!(
        harness.service.resume(id(1), id(11)).await.unwrap(),
        StartOutcome::Waiting(WaitReason::SizeUnavailable)
    );
    assert_eq!(harness.start(id(11)), pending);
    assert_eq!(provider.creates.lock().unwrap().len(), 0);
}
