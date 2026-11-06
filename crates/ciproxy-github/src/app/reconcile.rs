//! Leased repository policy and stable organization-label resource bindings.

use ciproxy_store_interface::github_lifecycle::GithubPreparedResources;
use ciproxy_store_interface::labels::Label;
use ciproxy_store_interface::organizations::{AppState, Organization, RepositoryAccess};
use ciproxy_store_interface::scale_sets::ScaleSet;

use crate::error::{Error, Result};
use crate::persistence::mutation::MutationLease;
use crate::scale_sets::ScaleSetApi;

use super::AppService;

impl AppService {
    pub(super) async fn refresh(&self, installation_id: i64) -> Result<()> {
        let Some(locked) = self.locked_registration(installation_id).await? else {
            return Ok(());
        };
        let Some(organization) = locked.organization else {
            self.deliver_lifecycle(locked.current.account.id, &*locked.lease)
                .await?;
            return Ok(());
        };
        if organization.app_state == AppState::Removed {
            self.deliver_lifecycle(organization.github_id, &*locked.lease)
                .await?;
            return Ok(());
        }
        self.synchronize(organization, locked.current, None, false, &*locked.lease)
            .await?;
        Ok(())
    }

    pub(super) async fn change_access(
        &self,
        installation_id: i64,
        access: RepositoryAccess,
    ) -> Result<()> {
        let Some(locked) = self.locked_registration(installation_id).await? else {
            return Err(Error::OrganizationInactive);
        };
        let Some(organization) = locked.organization else {
            return Err(Error::OrganizationInactive);
        };
        if organization.app_state == AppState::Removed {
            self.deliver_lifecycle(organization.github_id, &*locked.lease)
                .await?;
            return Err(Error::OrganizationInactive);
        }
        self.synchronize(
            organization,
            locked.current,
            Some(access),
            false,
            &*locked.lease,
        )
        .await?;
        Ok(())
    }

    pub(super) async fn prepare(
        &self,
        organization: &Organization,
        labels: Vec<Label>,
        lease: &dyn MutationLease,
    ) -> Result<GithubPreparedResources> {
        lease.check().await?;
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
        let existing = self.store.scale_sets(organization).await?;
        let saved_group = if let Some(set) = existing
            .iter()
            .find(|set| labels.iter().any(|label| label.id == set.label_id))
        {
            Some(set.runner_group_id)
        } else {
            self.store.runner_group(organization.github_id).await?
        };
        let group_name = format!("ciproxy-{}", organization.github_id);
        lease.check().await?;
        let group_id = self
            .rest
            .restrict_runner_group(
                organization.installation_id,
                &organization.login,
                &group_name,
                &allowed,
                saved_group,
            )
            .await?;
        lease.check().await?;
        let api = self.scale_sets.for_organization(organization).await?;
        release_retired(&*api, &existing, &labels, lease).await?;
        let mut scale_sets = Vec::new();
        for label in labels {
            let previous = existing.iter().find(|set| set.label_id == label.id);
            lease.check().await?;
            let github_id = api
                .ensure(&label.name, group_id, previous.map(|set| set.github_id))
                .await?;
            lease.check().await?;
            scale_sets.push(ScaleSet {
                id: previous
                    .map(|set| set.id)
                    .unwrap_or_else(|| self.runtime.uuid()),
                organization_id: organization.id,
                label_id: label.id,
                github_id,
                runner_group_id: group_id,
            });
        }
        Ok(GithubPreparedResources {
            repositories,
            scale_sets,
        })
    }
}

/// Release deleted-label names before a resume or reinstall prepares replacements.
async fn release_retired(
    api: &dyn ScaleSetApi,
    bindings: &[ScaleSet],
    labels: &[Label],
    lease: &dyn MutationLease,
) -> Result<()> {
    for binding in bindings
        .iter()
        .filter(|binding| !labels.iter().any(|label| label.id == binding.label_id))
    {
        lease.check().await?;
        api.rename_scale_set(
            binding.github_id,
            &format!("ciproxy-retired-{}", binding.id.simple()),
        )
        .await?;
        lease.check().await?;
    }
    Ok(())
}
