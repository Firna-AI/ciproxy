//! A retained complete runtime with the defined unknown start-fee accounting gap.

use ciproxy_store_interface::overflow::known::KnownSandbox;
use ciproxy_store_interface::overflow::record::{
    EncryptedLaunch, ProviderParameters, RuntimeCharge, RuntimeTotal, StartPhase, StartRecord,
};
use ciproxy_store_interface::secrets::TokenHash;
use rust_decimal::Decimal;
use uuid::Uuid;

use super::m18_support::{Pages, at};

pub(super) fn seed(pages: &Pages) -> Uuid {
    let store = &pages.fixture.database.store;
    let workspace = pages.fixture.workspace;
    let mut settings = store.settings(workspace).unwrap();
    settings.overflow_enabled = true;
    store.save_settings(workspace, &settings).unwrap();
    let scale = store
        .scale_sets(workspace, pages.fixture.organization.id)
        .unwrap()
        .remove(0);
    let label = store
        .labels(workspace)
        .unwrap()
        .into_iter()
        .find(|label| label.id == scale.label_id)
        .unwrap();
    let mut session = store.try_lock_overflow(workspace).unwrap().unwrap();
    let account = session.snapshot().unwrap().accounts.remove(0);
    let sizing = pages.services.boat_root_sizing.unwrap();
    let starts = at("2026-11-06T10:00:00Z");
    let ends = at("2026-11-06T11:00:00Z");
    let mut record = StartRecord {
        delivery_id: Uuid::new_v4(),
        runner_id: Uuid::new_v4(),
        organization_id: pages.fixture.organization.id,
        label_id: label.id,
        scale_set_id: scale.id,
        requested_vcpus: label.size.vcpus.try_into().unwrap(),
        requested_memory_gb: label.size.memory_gb.try_into().unwrap(),
        requested_disk_gb: label.size.disk_gb.try_into().unwrap(),
        image_root_bytes: sizing.image_root_bytes,
        required_root_disk_bytes: sizing.root_bytes(20_000_000_000).unwrap(),
        own_machine_fits: false,
        dispatch: None,
        key_id: account.account.id,
        deployment: "fit-test".into(),
        snapshot: "test-image".into(),
        phase: StartPhase::Unsent,
        sandbox: None,
        starts_at: starts,
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
    assert!(
        session
            .save_pending(
                &record,
                &EncryptedLaunch(vec![19]),
                &TokenHash::new(vec![23; 32])
            )
            .unwrap()
    );
    record.phase = StartPhase::Ended;
    record.sandbox = Some(KnownSandbox::observed("retained-gap".into(), starts, 21600));
    let mut size = account.state.sizes[0].clone();
    size.vcpus = 128;
    size.memory_bytes = 1_024_000_000_000;
    size.disk_bytes = 100_000_000_000;
    record.dispatch = Some(ProviderParameters {
        size,
        lifetime_seconds: 21600,
    });
    record.ends_at = Some(ends);
    record.runtime_complete = true;
    record.runtime = vec![RuntimeCharge {
        month: at("2026-11-01T00:00:00Z").date_naive(),
        usd: Decimal::ONE,
        until: ends,
        running: false,
    }];
    record.runtime_total = Some(RuntimeTotal {
        since: starts,
        until: ends,
        usd: Decimal::ONE,
        running: false,
    });
    session.save_start(&record).unwrap();
    record.delivery_id
}
