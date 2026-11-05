//! Public store fixtures for encrypted account and sandbox delivery records.

use ciproxy_store_interface::overflow::account::{AccountState, SizeOffer};
use ciproxy_store_interface::overflow::record::{StartPhase, StartRecord};
use ciproxy_store_interface::overflow::store::AccountRecord;
use ciproxy_store_interface::secrets::EncryptedBoatKey;
use rust_decimal::Decimal;
use uuid::Uuid;

use super::super::support::{Database, account, at};

pub(super) fn setup(database: &Database) -> (Uuid, AccountRecord, StartRecord) {
    let workspace = database.workspace();
    let organization = database.organization(&workspace);
    let (label, scale_set) = database.label_and_scale_set(&organization);
    let account = AccountRecord {
        account: account(),
        key: EncryptedBoatKey::new(vec![8, 9]),
        state: AccountState {
            wallet: "personal".to_owned(),
            checked_at: Some(at()),
            ..AccountState::default()
        },
    };
    let mut session = database
        .store
        .try_lock_overflow(workspace.id)
        .unwrap()
        .unwrap();
    session.connect(&account, 200_000_000).unwrap();
    let record = StartRecord {
        delivery_id: Uuid::new_v4(),
        runner_id: Uuid::new_v4(),
        organization_id: organization.id,
        label_id: label.id,
        scale_set_id: scale_set.id,
        requested_vcpus: 2,
        requested_memory_gb: 8,
        requested_disk_gb: 60,
        size: SizeOffer {
            name: "large".to_owned(),
            vcpus: 8,
            memory_bytes: 16_000_000_000,
            disk_bytes: 111_000_000_000,
            hourly_usd: Decimal::new(72, 3),
        },
        key_id: account.account.id,
        deployment: "ciproxy-test".to_owned(),
        snapshot: "shared-image".to_owned(),
        lifetime_seconds: 21600,
        phase: StartPhase::Creating,
        sandbox_id: None,
        starts_at: at(),
        ends_at: None,
        start_usd: Some(Decimal::new(3, 2)),
        reported_at: None,
        runtime: Vec::new(),
        runtime_complete: false,
    };
    (workspace.id, account, record)
}
