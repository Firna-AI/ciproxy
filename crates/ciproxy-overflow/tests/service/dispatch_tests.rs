//! Current size, lifetime, and budget commit together before the first create.

use chrono::Duration;
use ciproxy_overflow::api::{Overflow, StartOutcome, WaitReason};
use ciproxy_overflow::error::Error;
use ciproxy_scaler::actions::Destination;
use ciproxy_scaler::boat::BoatSizeId;
use ciproxy_store_interface::labels::WhenNothingFits;
use ciproxy_store_interface::overflow::record::StartPhase;
use rust_decimal::Decimal;
use sandbox_interface::{SandboxLifetime, SandboxSizeSelection};

use super::dispatch_support::Provider;
use super::fixtures::{at, id, request};
use super::harness::{Harness, Runtime};

#[tokio::test]
async fn resumed_fallback_selects_fitting_size_and_reserves_its_current_price() {
    let provider = Provider::new(false);
    let harness = Harness::new(provider.backend, (), Runtime::Start);
    let mut label = harness.database.store.labels(id(1)).unwrap().remove(0);
    label.size.vcpus = 16;
    label.size.memory_gb = 32;
    label.size.disk_gb = 180;
    label.when_nothing_fits = WhenNothingFits::LargestBoat;
    harness.database.store.update_label(id(1), &label).unwrap();
    let mut action = request();
    action.action.requested_size.vcpu = 16;
    action.action.requested_size.memory_bytes = 32_000_000_000;
    action.action.requested_size.disk_bytes = 180_000_000_000;
    harness.state.lock().unwrap().fail_on_key_open = Some(2);
    assert!(matches!(
        harness.service.start(id(1), action.clone()).await,
        Err(Error::KmsHttp { status: 503 })
    ));
    let pending = harness.start(id(11));
    assert_eq!(pending.phase, StartPhase::Unsent);
    assert_eq!(pending.dispatch, None);
    assert_eq!(pending.start_usd, None);
    assert_eq!(pending.billing, None);
    assert_eq!(
        harness
            .service
            .view(id(1))
            .await
            .unwrap()
            .spend
            .counted_usd()
            .unwrap(),
        Decimal::ZERO
    );
    let input = harness.database.session().encrypted_launch(id(11)).unwrap();
    let hash = harness.state.lock().unwrap().hashes[&id(11)].clone();
    *provider.sizes.lock().unwrap() = vec!["large".to_owned(), "xlarge".to_owned()];
    provider.facts.lock().unwrap().start_fee_bound_usd = Some(Decimal::ONE);
    *harness.now.lock().unwrap() = at() + Duration::minutes(5);
    harness.set_limit(500_000);
    assert_eq!(
        harness.service.resume(id(1), id(11)).await.unwrap(),
        StartOutcome::Waiting(WaitReason::SpendLimit {
            resumes_at: "2026-12-01T00:00:00Z".parse().unwrap(),
        })
    );
    assert_eq!(harness.start(id(11)), pending);
    assert_eq!(provider.creates.lock().unwrap().len(), 0);
    label.when_nothing_fits = WhenNothingFits::Wait;
    harness.database.store.update_label(id(1), &label).unwrap();
    harness.set_limit(1_230_000);
    assert!(matches!(
        harness.service.resume(id(1), id(11)).await.unwrap(),
        StartOutcome::Pending(_)
    ));
    let saved = harness.start(id(11));
    assert_eq!(saved.delivery_id, pending.delivery_id);
    assert_eq!(saved.runner_id, pending.runner_id);
    assert_eq!(saved.dispatch.as_ref().unwrap().size.name, "xlarge");
    assert_eq!(saved.dispatch.as_ref().unwrap().lifetime_seconds, 21600);
    assert_eq!(saved.start_fee_bound_usd, Some(Decimal::new(3, 2)));
    assert_eq!(saved.phase, StartPhase::Creating);
    assert_eq!(
        harness.database.session().encrypted_launch(id(11)).unwrap(),
        input
    );
    assert_eq!(
        harness
            .database
            .store
            .overflow_token_workspace(&hash)
            .unwrap(),
        Some(id(1))
    );
    assert_eq!(
        harness
            .state
            .lock()
            .unwrap()
            .budgets
            .last()
            .unwrap()
            .current_month_usd,
        Decimal::new(123, 2)
    );
    assert_eq!(
        provider.creates.lock().unwrap()[0].size,
        Some(SandboxSizeSelection::ProviderSize("xlarge".to_owned()))
    );
    action.action.destination = Destination::Boat(BoatSizeId("xlarge".to_owned()));
    action.action.size.vcpu = 16;
    action.action.size.memory_bytes = 32_000_000_000;
    action.action.size.disk_bytes = 251_000_000_000;
    assert!(matches!(
        harness.service.start(id(1), action).await.unwrap(),
        StartOutcome::Pending(_)
    ));
    assert_eq!(provider.creates.lock().unwrap().len(), 1);
    assert_eq!(
        provider.recoveries.lock().unwrap()[0].size,
        Some(SandboxSizeSelection::ProviderSize("xlarge".to_owned()))
    );
    assert_eq!(
        provider.recoveries.lock().unwrap()[0],
        provider.creates.lock().unwrap()[0]
    );

    let disk_provider = Provider::new(false);
    *disk_provider.sizes.lock().unwrap() = vec!["large".to_owned(), "xlarge".to_owned()];
    let disk_harness = Harness::new(disk_provider.backend, (), Runtime::Start);
    let mut disk_label = disk_harness.database.store.labels(id(1)).unwrap().remove(0);
    disk_label.size.vcpus = 2;
    disk_label.size.memory_gb = 8;
    disk_label.size.disk_gb = 63;
    disk_label.when_nothing_fits = WhenNothingFits::Wait;
    disk_harness
        .database
        .store
        .update_label(id(1), &disk_label)
        .unwrap();
    *disk_harness.now.lock().unwrap() = at() + Duration::minutes(5);
    let mut disk_action = request();
    disk_action.action.requested_size.vcpu = 2;
    disk_action.action.requested_size.memory_bytes = 8_000_000_000;
    disk_action.action.requested_size.disk_bytes = 63_000_000_000;
    assert!(matches!(
        disk_harness
            .service
            .start(id(1), disk_action)
            .await
            .unwrap(),
        StartOutcome::Pending(_)
    ));
    let disk_saved = disk_harness.start(id(11));
    assert_eq!(disk_saved.required_root_disk_bytes, 111_965_753_344);
    assert_eq!(disk_saved.dispatch.as_ref().unwrap().size.name, "xlarge");
    assert_eq!(
        disk_provider.creates.lock().unwrap()[0].size,
        Some(SandboxSizeSelection::ProviderSize("xlarge".to_owned()))
    );
}

#[tokio::test]
async fn paid_dispatch_replaces_trial_lifetime_and_keeps_fixed_recovery_parameters() {
    let provider = Provider::new(true);
    let harness = Harness::new(provider.backend, (), Runtime::Start);
    harness.state.lock().unwrap().fail_on_key_open = Some(3);
    assert!(matches!(
        harness.service.start(id(1), request()).await,
        Err(Error::KmsHttp { status: 503 })
    ));
    let pending = harness.start(id(11));
    provider.facts.lock().unwrap().trial = false;
    provider.facts.lock().unwrap().start_fee_bound_usd = Some(Decimal::new(7, 2));
    *harness.now.lock().unwrap() = at() + Duration::minutes(5);
    harness.set_limit(300_000);
    assert_eq!(
        harness.service.resume(id(1), id(11)).await.unwrap(),
        StartOutcome::Waiting(WaitReason::SpendLimit {
            resumes_at: "2026-12-01T00:00:00Z".parse().unwrap(),
        })
    );
    assert_eq!(harness.start(id(11)), pending);
    assert_eq!(provider.creates.lock().unwrap().len(), 0);
    harness.set_limit(462_000);
    assert!(matches!(
        harness.service.resume(id(1), id(11)).await.unwrap(),
        StartOutcome::Pending(_)
    ));
    let saved = harness.start(id(11));
    assert_eq!(saved.delivery_id, pending.delivery_id);
    assert_eq!(saved.runner_id, pending.runner_id);
    assert_eq!(saved.dispatch.as_ref().unwrap().lifetime_seconds, 21600);
    assert_eq!(saved.start_fee_bound_usd, Some(Decimal::new(3, 2)));
    assert_eq!(
        harness
            .state
            .lock()
            .unwrap()
            .budgets
            .last()
            .unwrap()
            .current_month_usd,
        Decimal::new(462, 3)
    );
    let lifetime = SandboxLifetime::OneShot {
        max_lifetime: std::time::Duration::from_secs(21600),
    };
    assert_eq!(provider.creates.lock().unwrap()[0].lifetime, lifetime);
    provider.facts.lock().unwrap().trial = true;
    assert!(matches!(
        harness.service.resume(id(1), id(11)).await.unwrap(),
        StartOutcome::Pending(_)
    ));
    assert_eq!(provider.recoveries.lock().unwrap()[0].lifetime, lifetime);
    assert_eq!(
        provider.recoveries.lock().unwrap()[0]
            .operation_id
            .as_uuid(),
        id(11)
    );
    assert_eq!(
        provider.recoveries.lock().unwrap()[0],
        provider.creates.lock().unwrap()[0]
    );
    assert_eq!(provider.creates.lock().unwrap().len(), 1);
    assert_eq!(harness.start(id(11)), saved);
}

#[tokio::test]
async fn paid_retry_inside_catalog_window_uses_fresh_trial_facts() {
    let provider = Provider::new(true);
    let harness = Harness::new(provider.backend, (), Runtime::Start);
    harness.state.lock().unwrap().fail_on_key_open = Some(2);
    assert!(matches!(
        harness.service.start(id(1), request()).await,
        Err(Error::KmsHttp { status: 503 })
    ));
    let pending = harness.start(id(11));
    provider.facts.lock().unwrap().trial = false;
    *harness.now.lock().unwrap() = at() + Duration::seconds(60);
    harness.set_limit(300_000);
    assert_eq!(
        harness.service.resume(id(1), id(11)).await.unwrap(),
        StartOutcome::Waiting(WaitReason::SpendLimit {
            resumes_at: "2026-12-01T00:00:00Z".parse().unwrap(),
        })
    );
    assert_eq!(harness.start(id(11)), pending);
    assert_eq!(provider.creates.lock().unwrap().len(), 0);
    assert_eq!(*provider.reads.lock().unwrap(), (2, 0));
    harness.set_limit(462_000);
    assert!(matches!(
        harness.service.resume(id(1), id(11)).await.unwrap(),
        StartOutcome::Pending(_)
    ));
    assert_eq!(
        provider.creates.lock().unwrap()[0].lifetime,
        SandboxLifetime::OneShot {
            max_lifetime: std::time::Duration::from_secs(21600)
        }
    );
    assert_eq!(
        harness.start(id(11)).start_fee_bound_usd,
        pending.start_fee_bound_usd
    );
    assert_eq!(*provider.reads.lock().unwrap(), (3, 0));
}
