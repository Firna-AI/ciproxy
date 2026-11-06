//! Controlled GitHub and time boundaries around the real store and gateway.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use ciproxy_api::error::Result;
use ciproxy_api::identity::{Grant, Profile, UserIdentity};
use ciproxy_api::installations::InstallationAdministration;
use ciproxy_github::app::LifecycleOutput;
use ciproxy_github::app::webhook::{Account, AccountKind};
use ciproxy_github::installation_state::{InstallationApi, InstalledApp};
use ciproxy_github::rest::GitHubRest;
use ciproxy_github::scale_sets::{JitRegistration, MessageQueue, ScaleSetApi, ScaleSetFactory};
use ciproxy_store_interface::dashboard::records::Membership;
use ciproxy_store_interface::github::{Repository, RunFacts};
use ciproxy_store_interface::github_lifecycle::LifecycleEvent;
use ciproxy_store_interface::github_timing::JobTimes;
use ciproxy_store_interface::organizations::Organization;
use ciproxy_store_interface::store::Store;
use secrecy::{ExposeSecret, SecretString};

pub(super) struct Identity {
    pub(super) id: AtomicI64,
    pub(super) memberships: Mutex<HashMap<i64, Vec<Membership>>>,
    pub(super) fail: AtomicBool,
}
#[async_trait]
impl UserIdentity for Identity {
    fn authorization_url(&self, state: &str, challenge: &str) -> String {
        format!("https://github.example/authorize?state={state}&code_challenge={challenge}")
    }
    async fn exchange(&self, _: SecretString, _: SecretString) -> Result<Grant> {
        Ok(Grant {
            access_token: self.id.load(Ordering::SeqCst).to_string().into(),
            expires_at: None,
            refresh_token: None,
            refresh_expires_at: None,
        })
    }
    async fn refresh(&self, grant: Grant) -> Result<Grant> {
        Ok(grant)
    }
    async fn profile(&self, grant: &Grant) -> Result<Profile> {
        let id = grant.access_token.expose_secret().parse().unwrap();
        Ok(Profile {
            id,
            login: format!("person-{id}"),
            avatar_url: "https://github.example/avatar".to_owned(),
        })
    }
    async fn installation(&self, _: &Grant, _: i64) -> Result<Option<InstalledApp>> {
        Ok(None)
    }
    async fn memberships(&self, grant: &Grant) -> Result<Vec<Membership>> {
        if self.fail.load(Ordering::SeqCst) {
            return Err(ciproxy_api::error::Error::Unavailable);
        }
        let id = grant.access_token.expose_secret().parse::<i64>().unwrap();
        Ok(self
            .memberships
            .lock()
            .unwrap()
            .get(&id)
            .cloned()
            .unwrap_or_default())
    }
}

pub(super) struct Remote {
    pub(super) store: Arc<dyn Store>,
    pub(super) organizations: Mutex<Vec<Organization>>,
    pub(super) sets: Mutex<HashMap<(String, i64), i64>>,
    pub(super) next_id: AtomicI64,
    pub(super) events: Mutex<Vec<String>>,
    pub(super) fail_uninstall: AtomicBool,
    pub(super) fail_delete: AtomicBool,
    pub(super) fail_lifecycle: AtomicBool,
}
#[async_trait]
impl InstallationApi for Remote {
    async fn for_organization(&self, login: &str) -> ciproxy_github::error::Result<InstalledApp> {
        let organization = self
            .organizations
            .lock()
            .unwrap()
            .iter()
            .find(|organization| organization.login == login)
            .cloned()
            .unwrap();
        Ok(InstalledApp {
            id: organization.installation_id,
            suspended: false,
            account: Account {
                id: organization.github_id,
                login: organization.login,
                kind: AccountKind::Organization,
            },
        })
    }
    async fn by_id(&self, id: i64) -> ciproxy_github::error::Result<InstalledApp> {
        let organization = self
            .organizations
            .lock()
            .unwrap()
            .iter()
            .find(|organization| organization.installation_id == id)
            .cloned()
            .unwrap();
        self.for_organization(&organization.login).await
    }
}
#[async_trait]
impl GitHubRest for Remote {
    async fn job_times(
        &self,
        _: i64,
        _: &str,
        _: i64,
        _: i64,
    ) -> ciproxy_github::error::Result<Option<JobTimes>> {
        Ok(None)
    }
    async fn repositories(
        &self,
        installation: i64,
    ) -> ciproxy_github::error::Result<Vec<Repository>> {
        let Some(organization) = self
            .store
            .organization_by_installation_id(installation)
            .unwrap()
        else {
            return Ok(Vec::new());
        };
        Ok(self
            .store
            .repositories(organization.workspace_id, organization.id)
            .unwrap())
    }
    async fn restrict_runner_group(
        &self,
        _: i64,
        _: &str,
        _: &str,
        _: &[i64],
        saved: Option<i64>,
    ) -> ciproxy_github::error::Result<i64> {
        Ok(saved.unwrap_or(47))
    }
    async fn run_facts(&self, _: i64, _: &str, _: i64) -> ciproxy_github::error::Result<RunFacts> {
        Err(ciproxy_github::error::Error::InstallationPending)
    }
}
pub(super) struct Factory(pub(super) Arc<Remote>);
#[async_trait]
impl ScaleSetFactory for Factory {
    async fn for_organization(
        &self,
        _: &Organization,
    ) -> ciproxy_github::error::Result<Arc<dyn ScaleSetApi>> {
        Ok(self.0.clone())
    }
}
#[async_trait]
impl ScaleSetApi for Remote {
    async fn find_runner(&self, _: &str) -> ciproxy_github::error::Result<Option<i64>> {
        Ok(None)
    }
    async fn ensure(
        &self,
        name: &str,
        group: i64,
        saved: Option<i64>,
    ) -> ciproxy_github::error::Result<i64> {
        if let Some(id) = saved {
            return Ok(id);
        }
        Ok(*self
            .sets
            .lock()
            .unwrap()
            .entry((name.to_owned(), group))
            .or_insert_with(|| self.next_id.fetch_add(1, Ordering::SeqCst)))
    }
    async fn remove_scale_set(&self, id: i64) -> ciproxy_github::error::Result<()> {
        if self.fail_delete.swap(false, Ordering::SeqCst) {
            return Err(ciproxy_github::error::Error::AccessUnavailable(503));
        }
        self.events.lock().unwrap().push(format!("delete:{id}"));
        self.sets.lock().unwrap().retain(|_, value| *value != id);
        Ok(())
    }
    async fn rename_scale_set(&self, id: i64, name: &str) -> ciproxy_github::error::Result<()> {
        let mut sets = self.sets.lock().unwrap();
        let previous = sets
            .iter()
            .find(|(_, value)| **value == id)
            .map(|(key, _)| key.clone());
        if let Some((previous, group)) = previous {
            sets.remove(&(previous, group));
            sets.insert((name.to_owned(), group), id);
        }
        Ok(())
    }
    async fn open(&self, _: i64, _: &str) -> ciproxy_github::error::Result<Box<dyn MessageQueue>> {
        Err(ciproxy_github::error::Error::InstallationPending)
    }
    async fn register(&self, _: i64, _: &str) -> ciproxy_github::error::Result<JitRegistration> {
        Err(ciproxy_github::error::Error::InstallationPending)
    }
    async fn remove_runner(&self, _: i64) -> ciproxy_github::error::Result<()> {
        Ok(())
    }
}
#[async_trait]
impl InstallationAdministration for Remote {
    async fn installation(&self, id: i64) -> Result<Option<InstalledApp>> {
        Ok(self
            .organizations
            .lock()
            .unwrap()
            .iter()
            .find(|organization| organization.installation_id == id)
            .map(|organization| InstalledApp {
                id: organization.installation_id,
                account: Account {
                    id: organization.github_id,
                    login: organization.login.clone(),
                    kind: AccountKind::Organization,
                },
                suspended: organization.app_state
                    == ciproxy_store_interface::organizations::AppState::Suspended,
            }))
    }
    async fn pending_requests(
        &self,
        _: i64,
    ) -> Result<Vec<ciproxy_api::wire::access::PendingInstallation>> {
        Ok(Vec::new())
    }
    async fn exists(&self, id: i64) -> Result<bool> {
        Ok(!self
            .events
            .lock()
            .unwrap()
            .contains(&format!("uninstall:{id}")))
    }
    async fn uninstall(&self, id: i64) -> Result<()> {
        if self.fail_uninstall.swap(false, Ordering::SeqCst) {
            return Err(ciproxy_api::error::Error::Unavailable);
        }
        self.events.lock().unwrap().push(format!("uninstall:{id}"));
        Ok(())
    }
}
#[async_trait]
impl LifecycleOutput for Remote {
    async fn changed(&self, event: LifecycleEvent) -> ciproxy_github::error::Result<()> {
        if self.fail_lifecycle.swap(false, Ordering::SeqCst) {
            return Err(ciproxy_github::error::Error::AccessUnavailable(503));
        }
        self.events.lock().unwrap().push(format!(
            "lifecycle:{}",
            event.organization().installation_id
        ));
        Ok(())
    }
}
