//! One lease owns current-state verification and the complete policy mutation.

use ciproxy_store_interface::organizations::Organization;

use crate::error::{Error, Result};
use crate::installation_state::InstalledApp;
use crate::persistence::mutation::MutationLease;

use super::AppService;
use super::webhook::{Account, AccountKind};

pub(super) struct LockedOrganization {
    pub(super) lease: Box<dyn MutationLease>,
    pub(super) current: InstalledApp,
    pub(super) organization: Option<Organization>,
}

impl AppService {
    pub(super) async fn current_installation(
        &self,
        account: &Account,
        lease: &dyn MutationLease,
    ) -> Result<InstalledApp> {
        lease.check().await?;
        let current = self.installations.for_organization(&account.login).await?;
        lease.check().await?;
        if current.account.id != account.id {
            return Err(Error::InstallationPending);
        }
        Ok(current)
    }

    pub(super) async fn locked_registration(
        &self,
        installation_id: i64,
    ) -> Result<Option<LockedOrganization>> {
        let identity = self.installations.by_id(installation_id).await?;
        if identity.account.kind != AccountKind::Organization {
            return Ok(None);
        }
        let lease = self.leases.acquire(identity.account.id).await?;
        let current = self
            .current_installation(&identity.account, &*lease)
            .await?;
        if current.id != installation_id {
            return Ok(None);
        }
        let organization = self.store.by_installation(installation_id).await?;
        Ok(Some(LockedOrganization {
            lease,
            current,
            organization,
        }))
    }
}
