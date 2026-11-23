//! Service composition with real persistence and mocked external facts.

use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_overflow::api::{Overflow, OverflowConfig};
use ciproxy_overflow::boat::BoatFactory;
use ciproxy_overflow::error::Result;
use ciproxy_overflow::maintenance::MaintenanceCapabilities;
use ciproxy_overflow::provider::{
    Capabilities, ProviderAccountApi, ProviderFactory, RegistrationFiles, SandboxStart,
};
use ciproxy_overflow::registry::{ImageSource, ProviderRegistry};
use ciproxy_overflow::secrets::SecretBytes;
use ciproxy_overflow::service::{OverflowDependencies, OverflowService};
use ciproxy_overflow::view::{ProviderView, RefreshStatus};
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use ciproxy_store_interface::overflow::record::{EncryptedLaunch, StartPhase, StartRecord};
use ciproxy_store_interface::overflow::store::{AccountRecord, OverflowSnapshot};
use sandbox_interface::BackendCreateSandboxRequest;
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
    ExitReport,
    Authority,
    Start,
}

struct Factory {
    backend: Capabilities,
    capture: Arc<Mutex<Capture>>,
}

impl ProviderFactory for Factory {
    fn for_key_check(&self, key: &Secret) -> Result<Arc<dyn ProviderAccountApi>> {
        Ok(self.capabilities(key, "personal")?.account)
    }

    fn capabilities(&self, _: &Secret, wallet: &str) -> Result<Capabilities> {
        assert_eq!(wallet, "personal");
        self.capture.lock().unwrap().active_backends += 1;
        Ok(self.backend.clone())
    }

    fn maintenance(&self, _: &Secret, wallet: &str) -> Result<MaintenanceCapabilities> {
        assert_eq!(wallet, "personal");
        Ok(self.backend.maintenance())
    }

    fn create_request(&self, start: SandboxStart<'_>) -> BackendCreateSandboxRequest {
        BoatFactory::new().create_request(start)
    }

    fn registration_files(&self, delivery_id: Uuid) -> RegistrationFiles {
        BoatFactory::new().registration_files(delivery_id)
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
        Self::with_provider(Some(backend), extra, kind)
    }

    pub(super) fn without_provider(kind: Runtime) -> Self {
        Self::with_provider(None, (), kind)
    }

    fn with_provider(
        backend: Option<Capabilities>,
        extra: impl unimock::Clause,
        kind: Runtime,
    ) -> Self {
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
        let providers = ProviderRegistry::default();
        let providers = match backend {
            Some(backend) => providers.with_provider(
                OverflowProvider::Boat,
                Arc::new(Factory {
                    backend,
                    capture: state.clone(),
                }),
                ImageSource {
                    reference: "shared-image-v7".to_owned(),
                    root_sizing: RootSizing {
                        image_root_bytes: 47_000_000_000,
                    },
                },
            ),
            None => providers,
        };
        let service = OverflowService::new(
            OverflowConfig {
                deployment: "ciproxy-main".to_owned(),
                exit_report_url: Secret::new("https://app.ciproxy.dev/api/overflow/end"),
            },
            OverflowDependencies {
                store: Arc::new(RecordingStore {
                    inner: database.store.clone(),
                    capture: state.clone(),
                }),
                cipher: runtime.clone(),
                providers: Arc::new(providers),
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

    pub(super) async fn boat_view(&self) -> ProviderView {
        let view = self.service.view(id(1)).await.unwrap();
        let boat = view.provider(OverflowProvider::Boat).unwrap();
        assert_eq!(boat.refresh, RefreshStatus::Current);
        boat.clone()
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
        pending.reported_at = None;
        pending.ends_at = None;
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
        self.database
            .store
            .save_overflow_provider_spend_limit(id(1), OverflowProvider::Boat, Some(limit))
            .unwrap();
    }

    pub(super) fn connect_account(&self, account: &AccountRecord) {
        let limit = self.snapshot().provider_settings[&account.account.provider]
            .spend_limit_usd_micros
            .unwrap();
        let mut session = self.database.session();
        let grant = session
            .begin_connection(account.account.id)
            .unwrap()
            .unwrap();
        assert!(session.complete_connection(grant, account, limit).unwrap());
    }
}
