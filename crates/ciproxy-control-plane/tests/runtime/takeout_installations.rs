//! Typed App installation responses for scoped take-out tests.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use ciproxy_github::app::webhook::{Account, AccountKind};
use ciproxy_github::error::{Error, Result};
use ciproxy_github::installation_state::{InstallationApi, InstalledApp};

pub(super) struct Probe {
    pub(super) calls: Mutex<Vec<i64>>,
}

impl Probe {
    pub(super) fn present() -> Arc<Self> {
        Arc::new(Self {
            calls: Mutex::new(Vec::new()),
        })
    }
}

#[async_trait]
impl InstallationApi for Probe {
    async fn by_id(&self, id: i64) -> Result<InstalledApp> {
        self.calls.lock().unwrap().push(id);
        Ok(InstalledApp {
            id,
            account: Account {
                id: 42,
                login: "own-runtime".to_owned(),
                kind: AccountKind::Organization,
            },
            suspended: false,
        })
    }
    async fn for_organization(&self, _: &str) -> Result<InstalledApp> {
        Err(Error::OrganizationInactive)
    }
}
