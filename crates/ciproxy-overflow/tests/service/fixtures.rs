//! Literal persisted delivery fixtures and mocked provider capabilities.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use ciproxy_overflow::api::StartRequest;
use ciproxy_overflow::provider::{AccountFacts, Capabilities};
use ciproxy_protocol::identifiers::{LabelName, OrganizationId};
use ciproxy_protocol::resources::VmSize;
use ciproxy_protocol::secret::Secret;
use ciproxy_scaler::actions::{Destination, StartReason, StartRunner};
use ciproxy_scaler::boat::BoatSizeId;
use ciproxy_store_interface::boat::BoatAccount;
use ciproxy_store_interface::overflow::account::{AccountState, SizeOffer};
use ciproxy_store_interface::overflow::known::KnownSandbox;
use ciproxy_store_interface::overflow::record::{StartPhase, StartRecord};
use ciproxy_store_interface::overflow::store::{AccountRecord, OverflowSnapshot};
use ciproxy_store_interface::secrets::{EncryptedBoatKey, TokenHash};
use rust_decimal::Decimal;
use sha2::{Digest, Sha256};
use unimock::Unimock;
use uuid::Uuid;

pub(super) fn id(value: u128) -> Uuid {
    Uuid::from_u128(value)
}
pub(super) fn at() -> DateTime<Utc> {
    "2026-11-05T12:00:00Z".parse().unwrap()
}

pub(super) fn offer() -> SizeOffer {
    SizeOffer {
        name: "large".to_owned(),
        vcpus: 8,
        memory_bytes: 16_000_000_000,
        disk_bytes: 111_000_000_000,
        hourly_usd: Decimal::new(72, 3),
    }
}

pub(super) fn snapshot() -> OverflowSnapshot {
    OverflowSnapshot {
        accounts: vec![AccountRecord {
            account: BoatAccount {
                id: id(2),
                account_name: "Customer".to_owned(),
                connected_at: at(),
                retired_at: None,
                rejected_at: None,
            },
            key: EncryptedBoatKey::new(vec![7, 7]),
            state: AccountState {
                wallet: "personal".to_owned(),
                checked_at: Some(at()),
                sizes: vec![offer()],
                ..AccountState::default()
            },
        }],
        starts: Vec::new(),
        enabled: true,
        spend_limit_usd_micros: Some(200_000_000),
    }
}

pub(super) fn facts() -> AccountFacts {
    AccountFacts {
        name: "Customer".to_owned(),
        wallet: "personal".to_owned(),
        trial: false,
        billing_ready: true,
        start_fee_bound_usd: Some(Decimal::new(3, 2)),
    }
}

pub(super) fn request() -> StartRequest {
    StartRequest {
        delivery_id: id(11),
        runner_id: id(12),
        organization_id: id(3),
        label_id: id(4),
        scale_set_id: id(5),
        action: StartRunner {
            organization_id: OrganizationId("github-org-42".to_owned()),
            label: LabelName("ciproxy".to_owned()),
            requested_size: VmSize {
                vcpu: 2,
                memory_bytes: 8_000_000_000,
                disk_bytes: 60_000_000_000,
            },
            size: VmSize {
                vcpu: 8,
                memory_bytes: 16_000_000_000,
                disk_bytes: 111_000_000_000,
            },
            destination: Destination::Boat(BoatSizeId("large".to_owned())),
            reason: StartReason::NoEligibleMachine,
        },
        registration: Secret::new("one-use-jit-registration"),
    }
}

pub(super) fn record(phase: StartPhase) -> StartRecord {
    StartRecord {
        delivery_id: id(11),
        runner_id: id(12),
        organization_id: id(3),
        label_id: id(4),
        scale_set_id: id(5),
        requested_vcpus: 2,
        requested_memory_gb: 8,
        requested_disk_gb: 60,
        size: offer(),
        key_id: id(2),
        deployment: format!("ciproxy-main/{}", id(1)),
        snapshot: "shared-image-v7".to_owned(),
        lifetime_seconds: 21600,
        phase,
        sandbox: Some(KnownSandbox::observed(
            "bx_23456789".to_owned(),
            at(),
            21600,
        )),
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
    }
}

pub(super) fn capabilities(mock: Unimock) -> Capabilities {
    let mock = Arc::new(mock);
    Capabilities {
        lifecycle: mock.clone(),
        capacity: mock.clone(),
        files: mock.clone(),
        account: mock.clone(),
        launcher: mock.clone(),
        inspection: mock,
    }
}

pub(super) fn digest(token: &Secret) -> TokenHash {
    TokenHash::new(Sha256::digest(token.expose_secret().as_bytes()).to_vec())
}
