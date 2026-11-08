//! Keep saved fixture resources while new label validation uses the real SDK.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use ciproxy_github::app::AppService;
use ciproxy_github::error::Result;
use ciproxy_github::persistence::StoreAdapter;
use ciproxy_github::scale_sets::{JitRegistration, MessageQueue, ScaleSetApi, ScaleSetFactory};
use ciproxy_store_interface::organizations::Organization;

use super::super::fixture::Fixture;

/// Compose the production label service with the candidate SDK factory.
pub(super) fn compose(
    fixture: &mut Fixture,
    sdk: Arc<dyn ScaleSetFactory>,
) -> Arc<Mutex<Vec<String>>> {
    let labels = Arc::new(Mutex::new(Vec::new()));
    let factory: Arc<dyn ScaleSetFactory> = Arc::new(Factory {
        existing: fixture.state.scale_sets.clone(),
        sdk,
        labels: labels.clone(),
    });
    let persistence = Arc::new(StoreAdapter::new(
        fixture.database.store.clone(),
        fixture.runtime.clone(),
    ));
    fixture.app = Arc::new(AppService {
        leases: persistence.clone(),
        installations: fixture.remote.clone(),
        store: persistence.clone(),
        outbox: persistence,
        rest: fixture.remote.clone(),
        scale_sets: factory.clone(),
        authority: fixture.access.clone(),
        lifecycle: fixture.remote.clone(),
        runtime: fixture.runtime.clone(),
    });
    fixture.state.github = fixture.app.clone();
    fixture.state.scale_sets = factory;
    labels
}

struct Factory {
    existing: Arc<dyn ScaleSetFactory>,
    sdk: Arc<dyn ScaleSetFactory>,
    labels: Arc<Mutex<Vec<String>>>,
}

#[async_trait]
impl ScaleSetFactory for Factory {
    async fn for_organization(&self, organization: &Organization) -> Result<Arc<dyn ScaleSetApi>> {
        Ok(Arc::new(Api {
            existing: self.existing.for_organization(organization).await?,
            sdk: self.sdk.for_organization(organization).await?,
            labels: self.labels.clone(),
        }))
    }
}

struct Api {
    existing: Arc<dyn ScaleSetApi>,
    sdk: Arc<dyn ScaleSetApi>,
    labels: Arc<Mutex<Vec<String>>>,
}

#[async_trait]
impl ScaleSetApi for Api {
    async fn ensure(&self, label: &str, group: i64, saved: Option<i64>) -> Result<i64> {
        if saved.is_some() {
            return self.existing.ensure(label, group, saved).await;
        }
        self.labels.lock().unwrap().push(label.to_owned());
        self.sdk.ensure(label, group, saved).await
    }
    async fn open(&self, id: i64, owner: &str) -> Result<Box<dyn MessageQueue>> {
        self.sdk.open(id, owner).await
    }
    async fn register(&self, id: i64, name: &str) -> Result<JitRegistration> {
        self.sdk.register(id, name).await
    }
    async fn remove_runner(&self, id: i64) -> Result<()> {
        self.sdk.remove_runner(id).await
    }
    async fn find_runner(&self, name: &str) -> Result<Option<i64>> {
        self.sdk.find_runner(name).await
    }
    async fn remove_scale_set(&self, id: i64) -> Result<()> {
        self.sdk.remove_scale_set(id).await
    }
    async fn rename_scale_set(&self, id: i64, name: &str) -> Result<()> {
        self.sdk.rename_scale_set(id, name).await
    }
}
