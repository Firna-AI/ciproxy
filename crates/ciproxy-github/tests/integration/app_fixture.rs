//! Controlled API boundaries around real PostgreSQL App coordination.

use ciproxy_store_interface::github_lifecycle::LifecycleEvent;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ciproxy_github::app::webhook::{Account, AccountKind};
use ciproxy_github::app::{
    AppService, GitHubApp, InstallAuthority, InstallationDecision, LifecycleOutput,
};
use ciproxy_github::error::{Error, Result};
use ciproxy_github::installation_state::{InstallationApi, InstalledApp};
use ciproxy_github::persistence::StoreAdapter;
use ciproxy_github::rest::GitHubRest;
use ciproxy_github::runtime::Runtime;
use ciproxy_store_interface::github::{Repository, RunFacts};
use ciproxy_store_interface::installation_links::{InstallationClaim, InstallationRecovery};
use ciproxy_store_interface::store::Store;
use tokio::sync::oneshot;
use uuid::Uuid;

use super::resource_fixture::{ResourceFactory, Resources, seed_installation_choice};

pub(super) fn at() -> DateTime<Utc> {
    "2026-11-04T12:00:00Z".parse().unwrap()
}
pub(super) fn installed(id: i64, suspended: bool) -> InstalledApp {
    InstalledApp {
        id,
        account: Account {
            id: 17,
            login: "customer".to_owned(),
            kind: AccountKind::Organization,
        },
        suspended,
    }
}
pub(super) fn repositories() -> Vec<Repository> {
    vec![
        Repository {
            id: 23,
            full_name: "customer/private".to_owned(),
            private: true,
            default_branch: "old-default".to_owned(),
        },
        Repository {
            id: 29,
            full_name: "customer/other".to_owned(),
            private: true,
            default_branch: "main".to_owned(),
        },
    ]
}

pub(super) struct Authoritative {
    pub(super) current: Mutex<Option<InstalledApp>>,
    pub(super) reads: AtomicUsize,
}
#[async_trait]
impl InstallationApi for Authoritative {
    async fn by_id(&self, installation_id: i64) -> Result<InstalledApp> {
        self.current
            .lock()
            .unwrap()
            .clone()
            .filter(|current| current.id == installation_id)
            .ok_or(Error::InstallationPending)
    }
    async fn for_organization(&self, _: &str) -> Result<InstalledApp> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        self.current
            .lock()
            .unwrap()
            .clone()
            .ok_or(Error::InstallationPending)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Policy {
    pub(super) name: String,
    pub(super) ids: Vec<i64>,
    pub(super) saved_id: Option<i64>,
}
type ReadGate = (oneshot::Sender<()>, oneshot::Receiver<()>);

pub(super) struct Remote {
    pub(super) repositories: Mutex<Vec<Repository>>,
    pub(super) policies: Mutex<Vec<Policy>>,
    pub(super) group_id: AtomicI64,
    gate: Mutex<Option<ReadGate>>,
}
impl Remote {
    pub(super) fn hold_next_read(&self) -> (oneshot::Receiver<()>, oneshot::Sender<()>) {
        let (entered, ready) = oneshot::channel();
        let (release, wait) = oneshot::channel();
        *self.gate.lock().unwrap() = Some((entered, wait));
        (ready, release)
    }
}
#[async_trait]
impl GitHubRest for Remote {
    async fn job_times(
        &self,
        _installation_id: i64,
        _repository: &str,
        _run_id: i64,
        _runner_id: i64,
    ) -> Result<Option<ciproxy_store_interface::github_timing::JobTimes>> {
        Ok(None)
    }
    async fn repositories(&self, _: i64) -> Result<Vec<Repository>> {
        let repositories = self.repositories.lock().unwrap().clone();
        let gate = self.gate.lock().unwrap().take();
        if let Some((entered, wait)) = gate {
            entered.send(()).unwrap();
            wait.await.unwrap();
        }
        Ok(repositories)
    }
    async fn restrict_runner_group(
        &self,
        _: i64,
        _: &str,
        name: &str,
        repository_ids: &[i64],
        saved_id: Option<i64>,
    ) -> Result<i64> {
        self.policies.lock().unwrap().push(Policy {
            name: name.to_owned(),
            ids: repository_ids.to_vec(),
            saved_id,
        });
        Ok(self.group_id.load(Ordering::SeqCst))
    }
    async fn run_facts(&self, _: i64, _: &str, _: i64) -> Result<RunFacts> {
        Err(Error::AccessUnavailable(501))
    }
}

pub(super) struct Authority {
    store: Mutex<Option<Arc<dyn Store>>>,
    pub(super) target: Mutex<Option<Uuid>>,
    pub(super) requesters: Mutex<Vec<i64>>,
    pub(super) admins: Mutex<Vec<i64>>,
    pub(super) fail_next_admin: AtomicBool,
}
#[async_trait]
impl InstallAuthority for Authority {
    async fn installation_decision(
        &self,
        claim: &InstallationClaim,
    ) -> Result<InstallationDecision> {
        self.requesters.lock().unwrap().push(claim.requester_id);
        if let InstallationRecovery::Add { workspace_id } = claim.recovery() {
            let store = self.store.lock().unwrap().clone().unwrap();
            store.authorize_installation_claim(claim.id, true).unwrap();
            Ok(InstallationDecision::Target {
                workspace_id,
                admin_approved: true,
                billing_hold: false,
            })
        } else {
            Ok(if claim.recovery() == InstallationRecovery::Direct {
                InstallationDecision::Direct
            } else {
                InstallationDecision::AwaitingAssignment
            })
        }
    }
    async fn ensure_initial_admin(&self, _: Uuid, user: i64) -> Result<()> {
        self.admins.lock().unwrap().push(user);
        if self.fail_next_admin.swap(false, Ordering::SeqCst) {
            return Err(Error::AccessUnavailable(503));
        }
        Ok(())
    }
}

#[derive(Default)]
pub(super) struct Output {
    pub(super) accepted: Mutex<Vec<LifecycleEvent>>,
    pub(super) attempts: Mutex<Vec<LifecycleEvent>>,
    pub(super) fail_next: AtomicBool,
}
#[async_trait]
impl LifecycleOutput for Output {
    async fn changed(&self, event: LifecycleEvent) -> Result<()> {
        self.attempts.lock().unwrap().push(event.clone());
        if self.fail_next.swap(false, Ordering::SeqCst) {
            return Err(Error::AccessUnavailable(503));
        }
        self.accepted.lock().unwrap().push(event);
        Ok(())
    }
}
struct Inputs(AtomicU64);
impl Runtime for Inputs {
    fn now(&self) -> DateTime<Utc> {
        at()
    }
    fn uuid(&self) -> Uuid {
        Uuid::from_u128(u128::from(self.0.fetch_add(1, Ordering::SeqCst)))
    }
}

pub(super) struct Fixture {
    pub(super) state: Arc<Authoritative>,
    pub(super) remote: Arc<Remote>,
    pub(super) resources: Arc<Resources>,
    pub(super) authority: Arc<Authority>,
    pub(super) output: Arc<Output>,
    runtime: Arc<dyn Runtime>,
}
impl Fixture {
    pub(super) fn new() -> Self {
        Self {
            state: Arc::new(Authoritative {
                current: Mutex::new(Some(installed(19, false))),
                reads: AtomicUsize::new(0),
            }),
            remote: Arc::new(Remote {
                repositories: Mutex::new(repositories()),
                policies: Mutex::new(vec![]),
                group_id: AtomicI64::new(47),
                gate: Mutex::new(None),
            }),
            resources: Arc::new(Resources::new()),
            authority: Arc::new(Authority {
                store: Mutex::new(None),
                target: Mutex::new(None),
                requesters: Mutex::new(vec![]),
                admins: Mutex::new(vec![]),
                fail_next_admin: AtomicBool::new(false),
            }),
            output: Arc::new(Output::default()),
            runtime: Arc::new(Inputs(AtomicU64::new(100))),
        }
    }
    pub(super) fn app(&self, store: Arc<dyn Store>) -> Arc<dyn GitHubApp> {
        self.app_with_installations(store, self.state.clone())
    }

    pub(super) fn app_with_installations(
        &self,
        store: Arc<dyn Store>,
        installations: Arc<dyn InstallationApi>,
    ) -> Arc<dyn GitHubApp> {
        if let Some(workspace) = *self.authority.target.lock().unwrap() {
            let current = self.state.current.lock().unwrap().clone().unwrap();
            seed_installation_choice(store.as_ref(), &current, workspace, 41);
        }
        *self.authority.store.lock().unwrap() = Some(store.clone());
        let adapter = Arc::new(StoreAdapter::new(store, self.runtime.clone()));
        Arc::new(AppService {
            leases: adapter.clone(),
            installations,
            store: adapter.clone(),
            outbox: adapter,
            rest: self.remote.clone(),
            scale_sets: Arc::new(ResourceFactory(self.resources.clone())),
            authority: self.authority.clone(),
            lifecycle: self.output.clone(),
            runtime: self.runtime.clone(),
        })
    }
}
