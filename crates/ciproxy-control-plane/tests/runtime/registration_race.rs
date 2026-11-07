//! Deterministic binding changes around the production registration service.

use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use ciproxy_github::error::{Error, Result};
use ciproxy_github::persistence::StoreAdapter;
use ciproxy_github::persistence::session::ListenerTarget;
use ciproxy_github::registrations::{
    RegisteredRunner, RegistrationRequest, RegistrationRetirement, RegistrationService,
    RunnerRegistrations,
};
use ciproxy_github::scale_sets::{JitRegistration, MessageQueue, ScaleSetApi, ScaleSetFactory};
use ciproxy_store_interface::organizations::Organization;
use ciproxy_store_interface::scale_sets::ScaleSet;
use ciproxy_store_interface::store::Store;
use uuid::Uuid;

use super::overflow_support::Harness;

pub(super) struct Race {
    store: Arc<dyn Store>,
    binding: ScaleSet,
    before_context: AtomicBool,
    after_context: AtomicBool,
    next_runner: AtomicI64,
    pub(super) registered: Mutex<Vec<i64>>,
    workspace: Uuid,
}

impl Race {
    pub(super) fn install(harness: &mut Harness) -> Arc<Self> {
        let workspace = harness.fixture.workspace;
        let race = Arc::new(Self {
            store: harness.fixture.store.clone(),
            binding: harness
                .fixture
                .store
                .scale_sets(workspace, harness.fixture.organization)
                .unwrap()
                .remove(0),
            workspace,
            before_context: AtomicBool::new(true),
            after_context: AtomicBool::new(true),
            next_runner: AtomicI64::new(100),
            registered: Mutex::new(Vec::new()),
        });
        let service: Arc<dyn RunnerRegistrations> = Arc::new(RegistrationService {
            store: Arc::new(StoreAdapter::new(
                harness.fixture.store.clone(),
                harness.ports.clone(),
            )),
            scale_sets: Arc::new(Factory(race.clone())),
        });
        harness.use_registrations(Arc::new(ContextRace {
            race: race.clone(),
            service,
        }));
        race
    }

    fn replace(&self, github_id: i64) {
        let mut binding = self.binding.clone();
        binding.github_id = github_id;
        self.store
            .save_github_scale_set(self.workspace, &binding)
            .unwrap();
    }
}

struct ContextRace {
    race: Arc<Race>,
    service: Arc<dyn RunnerRegistrations>,
}

#[async_trait]
impl RunnerRegistrations for ContextRace {
    async fn create(&self, request: RegistrationRequest, name: &str) -> Result<RegisteredRunner> {
        if self.race.before_context.swap(false, Ordering::SeqCst) {
            self.race.replace(144);
        }
        self.service.create(request, name).await
    }
    async fn retire(
        &self,
        target: ListenerTarget,
        name: &str,
        id: Option<i64>,
    ) -> Result<RegistrationRetirement> {
        self.service.retire(target, name, id).await
    }
}

struct Factory(Arc<Race>);

#[async_trait]
impl ScaleSetFactory for Factory {
    async fn for_organization(&self, _: &Organization) -> Result<Arc<dyn ScaleSetApi>> {
        Ok(self.0.clone())
    }
}

#[async_trait]
impl ScaleSetApi for Race {
    async fn register(&self, id: i64, _: &str) -> Result<JitRegistration> {
        self.registered.lock().unwrap().push(id);
        if self.after_context.swap(false, Ordering::SeqCst) {
            self.replace(244);
        }
        Ok(JitRegistration {
            runner_id: self.next_runner.fetch_add(1, Ordering::SeqCst),
            configuration: "race-fixture".into(),
        })
    }
    async fn remove_runner(&self, _: i64) -> Result<()> {
        Ok(())
    }
    async fn find_runner(&self, _: &str) -> Result<Option<i64>> {
        Ok(None)
    }
    async fn ensure(&self, _: &str, _: i64, _: Option<i64>) -> Result<i64> {
        Err(Error::AccessUnavailable(501))
    }
    async fn open(&self, _: i64, _: &str) -> Result<Box<dyn MessageQueue>> {
        Err(Error::AccessUnavailable(501))
    }
    async fn remove_scale_set(&self, _: i64) -> Result<()> {
        Err(Error::AccessUnavailable(501))
    }
    async fn rename_scale_set(&self, _: i64, _: &str) -> Result<()> {
        Err(Error::AccessUnavailable(501))
    }
}
