//! Retained original requests written through own-machine and Boat store paths.

use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_protocol::resources::VmSize;
use ciproxy_store_interface::boat::BoatAccount;
use ciproxy_store_interface::overflow::account::SizeOffer;
use ciproxy_store_interface::overflow::known::KnownSandbox;
use ciproxy_store_interface::overflow::record::{
    EncryptedLaunch, ProviderParameters, StartPhase, StartRecord,
};
use ciproxy_store_interface::own_runners::OwnRunner;
use ciproxy_store_interface::runners::{Runner, RunnerPlace, RunnerState};
use ciproxy_store_interface::secrets::{EncryptedBoatKey, TokenHash};
use rust_decimal::Decimal;
use uuid::Uuid;

use super::m18_support::{Pages, at};

pub(super) fn own_runner(pages: &Pages, machine_id: Uuid, github: i64) -> Runner {
    let fixture = &pages.fixture;
    let store = &fixture.database.store;
    let scale = store
        .scale_sets(fixture.workspace, fixture.organization.id)
        .unwrap()
        .remove(0);
    let label = store
        .labels(fixture.workspace)
        .unwrap()
        .into_iter()
        .find(|label| label.id == scale.label_id)
        .unwrap();
    let runner = Runner {
        id: Uuid::new_v4(),
        organization_id: fixture.organization.id,
        label_id: label.id,
        scale_set_id: scale.id,
        github_id: Some(github),
        state: RunnerState::Busy,
        size: VmSize {
            vcpu: 2,
            memory_bytes: 8_000_000_000,
            disk_bytes: 58_300_000_001,
        },
        place: RunnerPlace::Machine {
            machine_id,
            first_slot: 2,
        },
    };
    store
        .create_own_runner(
            fixture.workspace,
            &OwnRunner {
                registration_scale_set_id: None,
                requested_size: label.size,
                slot_indices: vec![2],
                runner: runner.clone(),
                label_name: label.name,
                command_id: Uuid::new_v4(),
                issued_at: at("2026-11-06T11:58:00Z"),
                acknowledged: true,
                confirmed_at: Some(at("2026-11-06T11:58:00Z")),
            },
        )
        .unwrap();
    runner
}

pub(super) fn boat_runner(pages: &Pages, github: i64) -> Runner {
    let fixture = &pages.fixture;
    let store = &fixture.database.store;
    let workspace = fixture.workspace;
    let mut settings = store.settings(workspace).unwrap();
    settings.overflow_enabled = true;
    store.save_settings(workspace, &settings).unwrap();
    let key_id = Uuid::new_v4();
    store
        .connect_boat(
            workspace,
            &BoatAccount {
                id: key_id,
                account_name: "retained-request".into(),
                connected_at: at("2026-11-01T00:00:00Z"),
                retired_at: None,
                rejected_at: None,
            },
            &EncryptedBoatKey::new(vec![17]),
            1_000_000,
        )
        .unwrap();
    let scale = store
        .scale_sets(workspace, fixture.organization.id)
        .unwrap()
        .remove(0);
    let label = store
        .labels(workspace)
        .unwrap()
        .into_iter()
        .find(|label| label.id == scale.label_id)
        .unwrap();
    let sizing = RootSizing {
        image_root_bytes: 58_300_000_000,
    };
    let starts_at = at("2026-11-06T11:59:00Z");
    let mut record = StartRecord {
        delivery_id: Uuid::new_v4(),
        runner_id: Uuid::new_v4(),
        organization_id: fixture.organization.id,
        label_id: label.id,
        scale_set_id: scale.id,
        requested_vcpus: label.size.vcpus.try_into().unwrap(),
        requested_memory_gb: label.size.memory_gb.try_into().unwrap(),
        requested_disk_gb: label.size.disk_gb.try_into().unwrap(),
        image_root_bytes: sizing.image_root_bytes,
        required_root_disk_bytes: sizing.root_bytes(20_000_000_000).unwrap(),
        own_machine_fits: true,
        dispatch: None,
        key_id,
        deployment: "retained-request".into(),
        snapshot: "test-image".into(),
        phase: StartPhase::Unsent,
        sandbox: None,
        starts_at,
        ends_at: None,
        billing: None,
        start_fee_bound_usd: None,
        start_usd: None,
        reported_at: None,
        runtime: Vec::new(),
        runtime_total: None,
        running_observed_at: None,
        runtime_complete: false,
    };
    {
        let mut session = store.try_lock_overflow(workspace).unwrap().unwrap();
        assert!(
            session
                .save_pending(
                    &record,
                    &EncryptedLaunch(vec![19]),
                    &TokenHash::new(vec![23; 32]),
                )
                .unwrap()
        );
        record.phase = StartPhase::Ended;
        record.sandbox = Some(KnownSandbox::observed(
            "retained-request".into(),
            starts_at,
            3600,
        ));
        record.dispatch = Some(ProviderParameters {
            size: SizeOffer {
                name: "retained-large".into(),
                vcpus: 4,
                memory_bytes: 16_000_000_000,
                disk_bytes: 100_000_000_000,
                hourly_usd: Decimal::ONE,
            },
            lifetime_seconds: 3600,
        });
        record.ends_at = Some(at("2026-11-06T12:00:00Z"));
        session.save_start(&record).unwrap();
    }
    store
        .update_runner(
            workspace,
            fixture.organization.id,
            record.runner_id,
            RunnerState::Stopped,
            Some(github),
        )
        .unwrap();
    store
        .runners(workspace, fixture.organization.id)
        .unwrap()
        .into_iter()
        .find(|runner| runner.id == record.runner_id)
        .unwrap()
}
