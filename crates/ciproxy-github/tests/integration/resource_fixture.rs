//! Typed remote scale-set controls for App service integration scenarios.

use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use ciproxy_github::error::{Error, Result};
use ciproxy_github::scale_sets::{JitRegistration, MessageQueue, ScaleSetApi, ScaleSetFactory};
use ciproxy_store_interface::billing::state::Tier;
use ciproxy_store_interface::dashboard::records::{Membership, User};
use ciproxy_store_interface::organizations::Organization;
use ciproxy_store_interface::organizations::{AppState, RepositoryAccess};
use ciproxy_store_interface::store::Store;
use uuid::Uuid;

use super::app_fixture::at;

/// The explicit-target scenarios need actual current membership, Admin, and paid capacity.
pub(super) fn seed_target_authority(store: &dyn Store, workspace: Uuid, requester: i64) {
    let organization = Organization {
        id: Uuid::new_v4(),
        workspace_id: workspace,
        github_id: 5000,
        login: "existing-target-member".to_owned(),
        installation_id: 5001,
        app_state: AppState::Active,
        repositories: RepositoryAccess::AllPrivate,
        removed_at: None,
        delete_after: None,
    };
    store.install_organization(&organization).unwrap();
    store
        .ensure_dashboard_initial_admin(workspace, requester, at())
        .unwrap();
    store
        .save_dashboard_user(
            &User {
                id: Uuid::new_v4(),
                github_id: requester,
                login: "target-admin".to_owned(),
                avatar_url: "https://github.example/avatar".to_owned(),
                credentials: vec![1],
                checked_at: at(),
            },
            &[Membership {
                github_id: organization.github_id,
                owner: false,
            }],
        )
        .unwrap();
    let mut billing = store.try_lock_billing(workspace).unwrap().unwrap();
    let mut record = billing.record().unwrap().state;
    record.tier = Tier::Team;
    billing
        .commit(Some(&record), None, false, false, at())
        .unwrap();
}

pub(super) struct Resources {
    pub(super) base_id: AtomicI64,
    pub(super) ensures: Mutex<Vec<(String, i64, Option<i64>)>>,
    pub(super) fail_next: AtomicBool,
}
impl Resources {
    pub(super) fn new() -> Self {
        Self {
            base_id: AtomicI64::new(53),
            ensures: Mutex::new(vec![]),
            fail_next: AtomicBool::new(false),
        }
    }
}
#[async_trait]
impl ScaleSetApi for Resources {
    async fn find_runner(&self, _name: &str) -> Result<Option<i64>> {
        Ok(None)
    }
    async fn ensure(&self, label: &str, group: i64, saved: Option<i64>) -> Result<i64> {
        if self.fail_next.swap(false, Ordering::SeqCst) {
            return Err(Error::AccessUnavailable(503));
        }
        self.ensures
            .lock()
            .unwrap()
            .push((label.to_owned(), group, saved));
        match label {
            "ciproxy" => Ok(self.base_id.load(Ordering::SeqCst)),
            "ciproxy-4vcpu" => Ok(59),
            "ciproxy-8vcpu" => Ok(61),
            _ => Err(Error::AccessUnavailable(422)),
        }
    }
    async fn open(&self, _: i64, _: &str) -> Result<Box<dyn MessageQueue>> {
        Err(Error::AccessUnavailable(501))
    }
    async fn register(&self, _: i64, _: &str) -> Result<JitRegistration> {
        Err(Error::AccessUnavailable(501))
    }
    async fn remove_runner(&self, _: i64) -> Result<()> {
        Err(Error::AccessUnavailable(501))
    }
    async fn remove_scale_set(&self, _: i64) -> Result<()> {
        Err(Error::AccessUnavailable(501))
    }
    async fn rename_scale_set(&self, _: i64, _: &str) -> Result<()> {
        Err(Error::AccessUnavailable(501))
    }
}

pub(super) struct ResourceFactory(pub(super) Arc<Resources>);

#[async_trait]
impl ScaleSetFactory for ResourceFactory {
    async fn for_organization(&self, _: &Organization) -> Result<Arc<dyn ScaleSetApi>> {
        Ok(self.0.clone())
    }
}
