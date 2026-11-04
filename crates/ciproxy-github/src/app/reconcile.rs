//! Visible private repositories, default branches, and one set per label.

use ciproxy_store_interface::organizations::{AppState, Organization, RepositoryAccess};
use ciproxy_store_interface::scale_sets::ScaleSet;

use crate::error::{Error, Result};

use super::AppService;

impl AppService {
    pub(super) async fn refresh(&self, installation_id: i64) -> Result<()> {
        let Some(organization) = self.store.by_installation(installation_id).await? else {
            return Ok(());
        };
        if organization.app_state != AppState::Active {
            return Ok(());
        }
        self.reconcile(&organization).await
    }

    pub(super) async fn change_access(
        &self,
        installation_id: i64,
        access: RepositoryAccess,
    ) -> Result<()> {
        let Some(mut organization) = self.store.by_installation(installation_id).await? else {
            return Err(Error::OrganizationInactive);
        };
        if organization.app_state != AppState::Active {
            return Err(Error::OrganizationInactive);
        }
        organization.repositories = access.clone();
        self.reconcile(&organization).await?;
        self.store.save_access(&organization, access).await
    }

    pub(super) async fn reconcile(&self, organization: &Organization) -> Result<()> {
        let repositories = self.rest.repositories(organization.installation_id).await?;
        let allowed: Vec<i64> = repositories
            .iter()
            .filter(|repository| repository.private)
            .filter(|repository| match &organization.repositories {
                RepositoryAccess::AllPrivate => true,
                RepositoryAccess::Selected(ids) => ids.contains(&repository.id),
            })
            .map(|repository| repository.id)
            .collect();
        let group_name = format!("ciproxy-{}", organization.id);
        let group_id = self
            .rest
            .restrict_runner_group(
                organization.installation_id,
                &organization.login,
                &group_name,
                &allowed,
            )
            .await?;
        self.store
            .save_repositories(organization, repositories)
            .await?;
        let existing = self.store.scale_sets(organization).await?;
        let api = self.scale_sets.for_organization(organization).await?;
        for label in self.store.labels(organization.workspace_id).await? {
            let github_id = api.ensure(&label.name, group_id).await?;
            if existing.iter().any(|set| set.label_id == label.id) {
                continue;
            }
            self.store
                .create_scale_set(
                    organization.workspace_id,
                    ScaleSet {
                        id: self.runtime.uuid(),
                        organization_id: organization.id,
                        label_id: label.id,
                        github_id,
                        runner_group_id: group_id,
                    },
                )
                .await?;
        }
        Ok(())
    }
}
