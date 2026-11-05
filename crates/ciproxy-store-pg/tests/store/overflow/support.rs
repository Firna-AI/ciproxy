//! Public store fixtures for encrypted account and sandbox delivery records.

use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::overflow::account::{AccountState, SizeOffer};
use ciproxy_store_interface::overflow::admission::{
    PreparedDispatch, ReservationBudget, ReservationOutcome,
};
use ciproxy_store_interface::overflow::record::{
    EncryptedLaunch, ProviderParameters, StartPhase, StartRecord,
};
use ciproxy_store_interface::overflow::store::AccountRecord;
use ciproxy_store_interface::overflow::store::OverflowSession;
use ciproxy_store_interface::secrets::EncryptedBoatKey;
use ciproxy_store_interface::secrets::TokenHash;
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
    let grant = session
        .begin_connection(account.account.id)
        .unwrap()
        .unwrap();
    assert!(
        session
            .complete_connection(grant, &account, 200_000_000)
            .unwrap()
    );
    let record = StartRecord {
        delivery_id: Uuid::new_v4(),
        runner_id: Uuid::new_v4(),
        organization_id: organization.id,
        label_id: label.id,
        scale_set_id: scale_set.id,
        requested_vcpus: 2,
        requested_memory_gb: 8,
        requested_disk_gb: 60,
        own_machine_fits: false,
        dispatch: Some(ProviderParameters {
            size: SizeOffer {
                name: "large".to_owned(),
                vcpus: 8,
                memory_bytes: 16_000_000_000,
                disk_bytes: 111_000_000_000,
                hourly_usd: Decimal::new(72, 3),
            },
            lifetime_seconds: 21600,
        }),
        key_id: account.account.id,
        deployment: "ciproxy-test".to_owned(),
        snapshot: "shared-image".to_owned(),
        phase: StartPhase::Creating,
        sandbox: None,
        starts_at: at(),
        ends_at: None,
        billing: None,
        start_fee_bound_usd: Some(Decimal::new(3, 2)),
        start_usd: Some(Decimal::new(3, 2)),
        reported_at: None,
        runtime: Vec::new(),
        runtime_total: None,
        running_observed_at: None,
        runtime_complete: false,
    };
    (workspace.id, account, record)
}

/// Seed a fixed provider operation through pending acceptance and atomic dispatch.
pub(super) fn reserve(
    session: &mut dyn OverflowSession,
    record: &StartRecord,
    launch: &EncryptedLaunch,
    hash: &TokenHash,
    budget: &ReservationBudget,
) -> Result<ReservationOutcome> {
    let mut pending = record.clone();
    pending.phase = StartPhase::Unsent;
    pending.dispatch = None;
    pending.sandbox = None;
    if !session.save_pending(&pending, launch, hash)? {
        return Ok(ReservationOutcome::NotPermitted);
    }
    if let Some(parameters) = &record.dispatch {
        let dispatch = PreparedDispatch {
            parameters: parameters.clone(),
            starts_at: record.starts_at,
            budget: budget.clone(),
        };
        let outcome = session.begin_dispatch(&pending, &dispatch)?;
        if outcome != ReservationOutcome::Reserved {
            return Ok(outcome);
        }
    }
    session.save_start(record)?;
    Ok(ReservationOutcome::Reserved)
}
