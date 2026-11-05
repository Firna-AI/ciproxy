//! Literal workspace fixtures and one capability-mocked service composition.

use std::{
    collections::{BTreeMap, VecDeque},
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Utc};
use ciproxy_protocol::identifiers::{LabelName, OrganizationId};
use ciproxy_protocol::resources::VmSize;
use ciproxy_protocol::secret::Secret;
use ciproxy_scaler::actions::{Destination, StartReason, StartRunner};
use ciproxy_scaler::boat::BoatSizeId;
use ciproxy_store_interface::boat::BoatAccount;
use ciproxy_store_interface::overflow::account::{AccountState, SizeOffer};
use ciproxy_store_interface::overflow::known::KnownSandbox;
use ciproxy_store_interface::overflow::record::{EncryptedLaunch, StartPhase, StartRecord};
use ciproxy_store_interface::overflow::store::{AccountRecord, OverflowSnapshot};
use ciproxy_store_interface::secrets::{EncryptedBoatKey, TokenHash};
use rust_decimal::Decimal;
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

use crate::api::{OverflowConfig, StartRequest};
use crate::clock::ClockMock;
use crate::crypto::{KeyCipherMock, KeyContext, SecretPurpose};
use crate::provider::{AccountFacts, BackendFactoryMock, Capabilities};
use crate::secrets::{SecretBytes, TokenSourceMock};
use crate::service::{OverflowDependencies, OverflowService};

use super::persistence;

#[derive(Default)]
pub(super) struct State {
    pub(super) snapshot: Option<OverflowSnapshot>,
    pub(super) launches: BTreeMap<Uuid, EncryptedLaunch>,
    pub(super) hashes: BTreeMap<Uuid, TokenHash>,
    pub(super) sealed: BTreeMap<(Uuid, u8), SecretBytes>,
    pub(super) contexts: Vec<KeyContext>,
    pub(super) phases: Vec<StartPhase>,
    pub(super) end_claims: VecDeque<Option<StartRecord>>,
    pub(super) token_lookups: VecDeque<Option<Uuid>>,
}

pub(super) struct Harness {
    pub(super) state: Arc<Mutex<State>>,
    pub(super) now: Arc<Mutex<DateTime<Utc>>>,
    pub(super) service: OverflowService,
}

impl Harness {
    pub(super) fn new(backend: Capabilities, extra: impl unimock::Clause, calls: &[&str]) -> Self {
        let state = Arc::new(Mutex::new(State {
            snapshot: Some(snapshot()),
            ..State::default()
        }));
        let now = Arc::new(Mutex::new(at()));
        let persistence = persistence::store(state.clone(), calls);
        let clock_now = now.clone();
        let sealing = state.clone();
        let opening = state.clone();
        let now_clause = ClockMock::now
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| *clock_now.lock().unwrap()))
            .at_least_times(1);
        let token = TokenSourceMock::token
            .some_call(matching!())
            .returns(Secret::new("callback-token-for-one-sandbox"))
            .at_least_times(1);
        let backend = BackendFactoryMock::backend
            .some_call(matching!(_, "personal"))
            .answers_arc(Arc::new(move |_, _, _| Ok(backend.clone())))
            .at_least_times(1);
        let seal = KeyCipherMock::seal
            .some_call(matching!(_, _))
            .answers_arc(Arc::new(move |_, context, bytes| {
                let purpose = match context.purpose {
                    SecretPurpose::BoatKey => 1,
                    SecretPurpose::Launch => 2,
                };
                let mut state = sealing.lock().unwrap();
                state.contexts.push(context);
                state.sealed.insert((context.id, purpose), bytes);
                Ok([context.id.as_bytes().as_slice(), &[purpose]].concat())
            }))
            .at_least_times(1);
        let open = KeyCipherMock::open
            .some_call(matching!(_, _))
            .answers_arc(Arc::new(move |_, context, _| {
                if context.purpose == SecretPurpose::BoatKey {
                    return Ok(SecretBytes::new(b"account-key".to_vec()));
                }
                Ok(opening
                    .lock()
                    .unwrap()
                    .sealed
                    .get(&(context.id, 2))
                    .expect("sealed launch")
                    .clone())
            }))
            .at_least_times(1);
        let methods: Vec<_> = ["now", "token", "backend", "seal", "open"]
            .into_iter()
            .filter(|method| calls.contains(method))
            .collect();
        let runtime = Arc::new(match methods.as_slice() {
            ["backend"] => Unimock::new((extra, backend)),
            ["backend", "open"] => Unimock::new((extra, backend, open)),
            ["now", "backend", "open"] => Unimock::new((extra, now_clause, backend, open)),
            ["now", "backend", "seal"] => Unimock::new((extra, now_clause, backend, seal)),
            ["now", "token", "backend", "seal", "open"] => {
                Unimock::new((extra, now_clause, token, backend, seal, open))
            }
            _ => panic!("unknown runtime fixture methods"),
        });
        let service = OverflowService::new(
            OverflowConfig {
                deployment: "ciproxy-main".to_owned(),
                shared_snapshot: "shared-image-v7".to_owned(),
                exit_report_url: Secret::new("https://app.ciproxy.dev/api/overflow/end"),
            },
            OverflowDependencies {
                store: persistence,
                cipher: runtime.clone(),
                factory: runtime.clone(),
                runners: runtime.clone(),
                clock: runtime.clone(),
                tokens: runtime,
            },
        );
        Self {
            state,
            now,
            service,
        }
    }
}

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
        next_start_usd: Some(Decimal::new(3, 2)),
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
        start_usd: Some(Decimal::new(3, 2)),
        reported_at: None,
        runtime: Vec::new(),
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
        launcher: mock,
    }
}
