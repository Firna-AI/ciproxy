//! One binding reconciliation path for label changes, activation, and retries.

use ciproxy_store_interface::labels::Label;
use ciproxy_store_interface::organizations::{AppState, Organization};
use ciproxy_store_interface::scale_sets::{ScaleSet, ScaleSetState};
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::persistence::mutation::MutationLease;
use crate::persistence::session::ListenerTarget;
use crate::scale_sets::ScaleSetApi;

use super::AppService;

impl AppService {
    pub(super) async fn validate_label(&self, workspace: Uuid, label: Label) -> Result<()> {
        let mut checked = false;
        for organization in self.store.organizations(workspace).await? {
            if organization.app_state != AppState::Active
                || !self.store.activation(&organization).await?.admitted
            {
                continue;
            }
            let lease = self.leases.acquire(organization.github_id).await?;
            let mut labels = self.store.labels(workspace).await?;
            labels.push(label.clone());
            self.prepare(&organization, labels, &*lease).await?;
            checked = true;
        }
        if !checked {
            return Err(Error::OrganizationInactive);
        }
        Ok(())
    }

    pub(super) async fn reconcile_workspace_labels(&self, workspace: Uuid) -> Result<()> {
        let mut failure = None;
        for organization in self.store.organizations(workspace).await? {
            if organization.app_state != AppState::Active {
                continue;
            }
            let result = if self.store.activation(&organization).await?.admitted {
                self.refresh(organization.installation_id).await
            } else {
                let lease = self.leases.acquire(organization.github_id).await?;
                let api = self.scale_sets.for_organization(&organization).await?;
                let labels = self.store.labels(workspace).await?;
                let bindings = self.store.scale_sets(&organization).await?;
                self.retire_bindings(&organization, &labels, &bindings, &*api, &*lease)
                    .await
            };
            if let Err(error) = result {
                failure = Some(error);
            }
        }
        if let Some(error) = failure {
            return Err(error);
        }
        Ok(())
    }

    pub(super) async fn reconcile_bindings(
        &self,
        organization: &Organization,
        labels: &[Label],
        bindings: &[ScaleSet],
        group_id: i64,
        api: &dyn ScaleSetApi,
        lease: &dyn MutationLease,
    ) -> Result<Vec<ScaleSet>> {
        self.retire_bindings(organization, labels, bindings, api, lease)
            .await?;
        let mut prepared = Vec::new();
        for label in labels {
            let previous = bindings.iter().find(|set| set.label_id == label.id);
            lease.check().await?;
            let github_id = api
                .ensure(&label.name, group_id, previous.map(|set| set.github_id))
                .await?;
            lease.check().await?;
            prepared.push(ScaleSet {
                id: previous.map_or_else(|| self.runtime.uuid(), |set| set.id),
                organization_id: organization.id,
                label_id: label.id,
                github_id,
                runner_group_id: group_id,
            });
        }
        Ok(prepared)
    }

    async fn retire_bindings(
        &self,
        organization: &Organization,
        labels: &[Label],
        bindings: &[ScaleSet],
        api: &dyn ScaleSetApi,
        lease: &dyn MutationLease,
    ) -> Result<()> {
        for set in bindings
            .iter()
            .filter(|set| !labels.iter().any(|label| label.id == set.label_id))
        {
            let target = ListenerTarget {
                workspace_id: organization.workspace_id,
                organization_id: organization.id,
                scale_set_id: set.id,
            };
            lease.check().await?;
            let Some(binding) = self.store.binding(target).await? else {
                continue;
            };
            if binding.state != ScaleSetState::Draining {
                continue;
            }
            let lock = if binding.has_tracked_jobs {
                None
            } else {
                self.store.lock_binding(target).await?
            };
            if let Some(lock) = lock {
                let Some(binding) = self.store.binding(target).await? else {
                    continue;
                };
                if binding.state != ScaleSetState::Draining {
                    continue;
                }
                if !binding.has_tracked_jobs {
                    lease.check().await?;
                    lock.check().await?;
                    api.remove_scale_set(binding.scale_set.github_id).await?;
                    lock.check().await?;
                    lease.check().await?;
                    self.store
                        .complete_binding(
                            organization.workspace_id,
                            binding.scale_set,
                            self.runtime.now(),
                        )
                        .await?;
                    continue;
                }
            }
            lease.check().await?;
            api.rename_scale_set(
                set.github_id,
                &format!("ciproxy-retired-{}", set.id.simple()),
            )
            .await?;
            lease.check().await?;
        }
        Ok(())
    }
}
