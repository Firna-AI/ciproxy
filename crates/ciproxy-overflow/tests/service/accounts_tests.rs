//! Saved credentials, five-minute catalogs, and the real scaler size policy.

use chrono::Duration;
use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_overflow::api::{Overflow, StartOutcome};
use ciproxy_overflow::crypto::SecretPurpose;
use ciproxy_overflow::error::Error;
use ciproxy_overflow::provider::ProviderAccountApiMock;
use ciproxy_protocol::identifiers::{LabelName, OrganizationId, WorkspaceId};
use ciproxy_protocol::resources::VmSize;
use ciproxy_protocol::secret::Secret;
use ciproxy_scaler::actions::{Action, Destination};
use ciproxy_scaler::decision::decide;
use ciproxy_scaler::provider::{ProviderAccount, ProviderId, ProviderSizeId};
use ciproxy_scaler::registration::RegistrationTarget;
use ciproxy_scaler::snapshot::{Label, NothingFits, Place, Snapshot, WaitingJob};
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use sandbox_interface::SandboxAccountCapacityMock;
use unimock::{MockFn, Unimock, matching};

use super::harness::{Harness, Runtime};

use super::fixtures::{at, capabilities, facts, id};
use ciproxy_protocol::platform::Platform;

#[tokio::test]
async fn rejected_key_is_never_encrypted_or_saved() {
    let backend = capabilities(Unimock::new(
        ProviderAccountApiMock::account
            .some_call(matching!())
            .returns(Err(Error::RejectedKey)),
    ));
    let harness = Harness::new(backend, (), Runtime::KeyCheck);
    assert!(matches!(
        harness
            .service
            .connect(
                id(1),
                OverflowProvider::Boat,
                id(8),
                Secret::new("invalid-key"),
                "personal".to_owned(),
                20_000_000
            )
            .await,
        Err(Error::RejectedKey)
    ));
    let state = harness.state.lock().unwrap();
    assert_eq!(harness.snapshot().accounts.len(), 1);
    assert_eq!(state.contexts, []);
}

#[tokio::test]
async fn accepted_key_is_workspace_bound_ciphertext_and_requires_a_positive_limit() {
    let backend = capabilities(Unimock::new((
        ProviderAccountApiMock::account
            .some_call(matching!())
            .answers(&|_| Ok(facts())),
        SandboxAccountCapacityMock::account_capacity
            .some_call(matching!())
            .returns(Ok(vec![sandbox_boat::catalogue::offer("large").unwrap()])),
    )));
    let harness = Harness::new(backend, (), Runtime::Connect);
    assert!(matches!(
        harness
            .service
            .connect(
                id(1),
                OverflowProvider::Boat,
                id(8),
                Secret::new("new-key"),
                "personal".to_owned(),
                0
            )
            .await,
        Err(Error::InvalidSpendLimit)
    ));
    let account = harness
        .service
        .connect(
            id(1),
            OverflowProvider::Boat,
            id(8),
            Secret::new("new-key"),
            "personal".to_owned(),
            20_000_000,
        )
        .await
        .unwrap();
    assert_eq!(account.account_name, "Customer");
    let state = harness.state.lock().unwrap();
    assert_eq!(state.contexts[0].workspace_id, id(1));
    assert_eq!(state.contexts[0].id, id(8));
    assert_eq!(state.contexts[0].purpose, SecretPurpose::BoatKey);
    assert_eq!(
        harness.snapshot().provider_settings[&OverflowProvider::Boat].spend_limit_usd_micros,
        Some(20_000_000)
    );
    assert_ne!(harness.snapshot().accounts[0].key.as_bytes(), b"new-key");
}

#[tokio::test]
async fn catalog_refreshes_at_five_minutes_and_drives_cheapest_or_largest_choices() {
    let backend = capabilities(Unimock::new((
        ProviderAccountApiMock::account
            .some_call(matching!())
            .answers(&|_| Ok(facts()))
            .once(),
        SandboxAccountCapacityMock::account_capacity
            .some_call(matching!())
            .returns(Ok(["small", "default", "large", "xlarge"]
                .map(|name| sandbox_boat::catalogue::offer(name).unwrap())
                .to_vec()))
            .once(),
    )));
    let harness = Harness::new(backend, (), Runtime::Read);
    *harness.now.lock().unwrap() = at() + Duration::seconds(299);
    assert_eq!(harness.boat_view().await.state.sizes.len(), 1);
    *harness.now.lock().unwrap() = at() + Duration::seconds(300);
    let view = harness.service.view(id(1)).await.unwrap();
    let boat = view.provider(OverflowProvider::Boat).unwrap();
    assert_eq!(boat.state.sizes.len(), 4);
    let label_name = LabelName("ciproxy".to_owned());
    let target = RegistrationTarget {
        organization_id: OrganizationId("github-org-42".to_owned()),
        label_id: id(4).to_string(),
        scale_set_id: id(5).to_string(),
        github_id: 53,
    };
    let provider = ProviderId::new(OverflowProvider::Boat.as_str());
    let root_sizing = RootSizing {
        image_root_bytes: 47_000_000_000,
    };
    let mut snapshot = Snapshot {
        workspace_id: WorkspaceId(id(1).to_string()),
        root_sizing,
        now: std::time::Duration::from_secs(300),
        control_plane_available: true,
        organizations: vec![OrganizationId("github-org-42".to_owned())],
        labels: vec![Label {
            platform: Platform::LinuxX64,
            targets: vec![target.clone()],
            name: label_name.clone(),
            size: VmSize {
                vcpu: 2,
                memory_bytes: 8_000_000_000,
                disk_bytes: 60_000_000_000,
            },
            order: vec![Place::OwnMachines, Place::Provider(provider)],
            nothing_fits: NothingFits::Wait,
        }],
        queue: vec![WaitingJob {
            target,
            queued_at: std::time::Duration::ZERO,
        }],
        machines: Vec::new(),
        overflow_enabled: view.enabled,
        providers: vec![ProviderAccount {
            id: provider,
            available: true,
            root_sizing,
            facts: boat.scaler_account(at()).unwrap(),
        }],
        sandboxes: Vec::new(),
        overflow_wait: std::time::Duration::from_secs(30),
    };
    let actions = decide(&snapshot);
    assert_eq!(actions.len(), 1);
    let Action::StartRunner(start) = &actions[0] else {
        panic!("start required");
    };
    assert_eq!(
        start.destination,
        Destination::Provider {
            provider,
            size_id: ProviderSizeId("large".to_owned())
        }
    );
    snapshot.labels[0].size.vcpu = 100;
    assert_eq!(decide(&snapshot), []);
    snapshot.labels[0].nothing_fits = NothingFits::LargestProviderSize;
    let actions = decide(&snapshot);
    let Action::StartRunner(start) = &actions[0] else {
        panic!("largest start required");
    };
    assert_eq!(
        start.destination,
        Destination::Provider {
            provider,
            size_id: ProviderSizeId("xlarge".to_owned())
        }
    );
    assert_eq!(start.size.disk_bytes, 251_000_000_000);
}

#[tokio::test]
async fn newly_rejected_saved_key_stops_starts_and_keeps_its_rejection_time() {
    let backend = capabilities(Unimock::new((
        ProviderAccountApiMock::account
            .some_call(matching!())
            .returns(Err(Error::RejectedKey)),
        ProviderAccountApiMock::usage
            .some_call(matching!(_, _, _))
            .answers(&|_, _, _, _| Err(Error::RejectedKey)),
    )));
    let harness = Harness::new(backend, (), Runtime::Read);
    harness.seed(super::coverage_tests::running_observation());
    *harness.now.lock().unwrap() = at() + Duration::minutes(5);
    let view = harness.boat_view().await;
    assert_eq!(
        view.account.unwrap().rejected_at,
        Some(at() + Duration::minutes(5))
    );
    assert_eq!(view.spend.unmeasured_runtime, [id(11)]);
    assert!(!harness.snapshot().starts[0].runtime_complete);
    let again = harness.boat_view().await;
    assert_eq!(
        again.account.unwrap().rejected_at,
        Some(at() + Duration::minutes(5))
    );

    let mut next = super::fixtures::request();
    next.delivery_id = id(21);
    next.runner_id = id(22);
    assert!(matches!(
        harness.service.start(id(1), next).await.unwrap(),
        StartOutcome::Waiting(ciproxy_overflow::api::WaitReason::Disconnected)
    ));
}
