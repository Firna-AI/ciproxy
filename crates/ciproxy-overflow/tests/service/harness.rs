//! Service composition with real persistence and mocked external facts.

use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_overflow::api::OverflowConfig;
use ciproxy_overflow::error::Result;
use ciproxy_overflow::maintenance::MaintenanceCapabilities;
use ciproxy_overflow::provider::{BackendFactory, Capabilities};
use ciproxy_overflow::secrets::SecretBytes;
use ciproxy_overflow::service::{OverflowDependencies, OverflowService};
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::overflow::record::{EncryptedLaunch, StartPhase, StartRecord};
use ciproxy_store_interface::overflow::store::{AccountRecord, OverflowSnapshot};
use uuid::Uuid;

use super::database::Database;
use super::fixtures::{at, digest, id, snapshot};
use super::recording::{Capture, RecordingStore};
use super::runtime;

pub(super) enum Runtime {
    Cancellation,
    KeyCheck,
    Connect,
    Recover,
    Read,
    Authority,
    Start,
}

struct Factory {
    backend: Capabilities,
    capture: Arc<Mutex<Capture>>,
}

impl BackendFactory for Factory {
    fn backend(&self, _: &Secret, wallet: &str) -> Result<Capabilities> {
        assert_eq!(wallet, "personal");
        self.capture.lock().unwrap().active_backends += 1;
        Ok(self.backend.clone())
    }

    fn maintenance(&self, _: &Secret, wallet: &str) -> Result<MaintenanceCapabilities> {
        assert_eq!(wallet, "personal");
        Ok(self.backend.maintenance())
    }
}

pub(super) struct Harness {
    pub(super) state: Arc<Mutex<Capture>>,
    pub(super) now: Arc<Mutex<DateTime<Utc>>>,
    pub(super) service: OverflowService,
    pub(super) database: Database,
}

impl Harness {
    pub(super) fn new(backend: Capabilities, extra: impl unimock::Clause, kind: Runtime) -> Self {
        let database = Database::new();
        {
            let mut session = database.session();
            let account = &snapshot().accounts[0];
            let grant = session
                .begin_connection(account.account.id)
                .unwrap()
                .unwrap();
            assert!(
                session
                    .complete_connection(grant, account, 200_000_000)
                    .unwrap()
            );
        }
        let state = Arc::new(Mutex::new(Capture::default()));
        let now = Arc::new(Mutex::new(at()));
        let runtime = runtime::build(extra, state.clone(), now.clone(), kind);
        let service = OverflowService::new(
            OverflowConfig {
                deployment: "ciproxy-main".to_owned(),
                shared_snapshot: "shared-image-v7".to_owned(),
                root_sizing: RootSizing {
                    image_root_bytes: 47_000_000_000,
                },
                exit_report_url: Secret::new("https://app.ciproxy.dev/api/overflow/end"),
            },
            OverflowDependencies {
                store: Arc::new(RecordingStore {
                    inner: database.store.clone(),
                    capture: state.clone(),
                }),
                cipher: runtime.clone(),
                factory: Arc::new(Factory {
                    backend,
                    capture: state.clone(),
                }),
                runners: runtime.clone(),
                clock: runtime.clone(),
                tokens: runtime,
            },
        );
        Self {
            state,
            now,
            service,
            database,
        }
    }

    pub(super) fn snapshot(&self) -> OverflowSnapshot {
        self.database.session().snapshot().unwrap()
    }

    pub(super) fn start(&self, delivery: Uuid) -> StartRecord {
        self.snapshot()
            .starts
            .into_iter()
            .find(|record| record.delivery_id == delivery)
            .unwrap()
    }

    pub(super) fn seed(&self, record: StartRecord) {
        let token = Secret::new(format!("seed-callback-{}", record.delivery_id));
        self.seed_token(record, &token);
    }

    pub(super) fn seed_token(&self, record: StartRecord, token: &Secret) {
        let mut session = self.database.session();
        let mut pending = record.clone();
        pending.phase = StartPhase::Unsent;
        pending.dispatch = None;
        pending.sandbox = None;
        assert!(
            session
                .save_pending(&pending, &EncryptedLaunch(vec![17]), &digest(token))
                .unwrap()
        );
        session.save_start(&record).unwrap();
        self.state
            .lock()
            .unwrap()
            .sealed
            .entry((record.delivery_id, 2))
            .or_insert_with(|| SecretBytes::new(vec![9]));
    }

    pub(super) fn save(&self, record: &StartRecord) {
        self.database.session().save_start(record).unwrap();
    }

    pub(super) fn save_account(&self, account: &AccountRecord) {
        self.database.session().save_account(account).unwrap();
    }

    pub(super) fn set_checked(&self, at: DateTime<Utc>) {
        let mut account = self
            .snapshot()
            .accounts
            .into_iter()
            .find(|account| account.account.retired_at.is_none())
            .unwrap();
        account.state.checked_at = Some(at);
        self.save_account(&account);
    }

    pub(super) fn set_limit(&self, limit: i64) {
        let mut settings = self.database.store.settings(id(1)).unwrap();
        settings.boat_spend_limit_usd_micros = Some(limit);
        self.database.store.save_settings(id(1), &settings).unwrap();
    }

    pub(super) fn connect_account(&self, account: &AccountRecord) {
        let limit = self.snapshot().spend_limit_usd_micros.unwrap();
        let mut session = self.database.session();
        let grant = session
            .begin_connection(account.account.id)
            .unwrap()
            .unwrap();
        assert!(session.complete_connection(grant, account, limit).unwrap());
    }
}
