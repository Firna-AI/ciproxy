//! One binding reconciliation path for label changes, activation, and retries.

use ciproxy_store_interface::labels::Label;
use ciproxy_store_interface::organizations::{AppState, Organization};
use ciproxy_store_interface::scale_sets::{ScaleSet, ScaleSetState};
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::persistence::mutation::MutationLease;
use crate::persistence::session::ListenerTarget;
use crate::scale_sets::ScaleSetApi;
use crate::scale_sets::ownership::{quarantine_name, retirement_name};

use super::AppService;

impl AppService {
    pub(super) async fn validate_label(&self, workspace: Uuid, label: Label) -> Result<()> {
        let mut checked = false;
        for candidate in self
            .store
            .organizations(workspace)
            .await?
            .into_iter()
            .filter(|organization| organization.app_state == AppState::Active)
        {
            let Some(locked) = self.locked_registration(candidate.installation_id).await? else {
                continue;
            };
            let Some(mut organization) = locked.organization else {
                continue;
            };
            if organization.workspace_id != workspace
                || organization.app_state != AppState::Active
                || locked.current.suspended
                || !self.store.activation(&organization).await?.admitted
            {
                continue;
            }
            organization.login = locked.current.account.login;
            let mut labels = self.store.labels(workspace).await?;
            labels.push(label.clone());
            self.prepare(&organization, labels, &*locked.lease).await?;
            checked = true;
        }
        if !checked {
            return Err(Error::OrganizationInactive);
        }
        Ok(())
    }

    pub(super) async fn reconcile_workspace_labels(&self, workspace: Uuid) -> Result<()> {
        let mut failure = None;
        for candidate in self
            .store
            .organizations(workspace)
            .await?
            .into_iter()
            .filter(|organization| organization.app_state == AppState::Active)
        {
            let locked = match self.locked_registration(candidate.installation_id).await {
                Ok(Some(locked)) => locked,
                Ok(None) => continue,
                Err(error) => {
                    failure = Some(error);
                    continue;
                }
            };
            let Some(mut organization) = locked.organization else {
                continue;
            };
            if organization.workspace_id != workspace || organization.app_state != AppState::Active
            {
                continue;
            }
            let result = if self.store.activation(&organization).await?.admitted {
                self.synchronize(organization, locked.current, None, false, &*locked.lease)
                    .await
                    .map(|_| ())
            } else {
                if locked.current.suspended {
                    continue;
                }
                organization.login = locked.current.account.login;
                let api = self.scale_sets.for_organization(&organization).await?;
                let labels = self.store.labels(workspace).await?;
                let bindings = self.store.scale_sets(&organization).await?;
                self.retire_bindings(&organization, &labels, &bindings, &*api, &*locked.lease)
                    .await
                    .map(|_| ())
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
        let foreign = self
            .retire_bindings(organization, labels, bindings, api, lease)
            .await?;
        let mut prepared = Vec::new();
        for label in labels {
            let previous = bindings.iter().find(|set| set.label_id == label.id);
            let reusable = previous.filter(|set| !foreign.contains(&set.github_id));
            lease.check().await?;
            let mut github_id = api
                .ensure(&label.name, group_id, reusable.map(|set| set.github_id))
                .await?;
            lease.check().await?;
            let mut attempt = 0;
            while foreign.contains(&github_id) {
                attempt += 1;
                let name =
                    retirement_name(&format!("ciproxy-retired-{github_id}-{attempt}"), labels);
                api.rename_scale_set(github_id, &name).await?;
                lease.check().await?;
                github_id = api.ensure(&label.name, group_id, None).await?;
                lease.check().await?;
            }
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
    ) -> Result<Vec<i64>> {
        let foreign = self
            .retire_foreign_bindings(organization, labels, bindings, api, lease)
            .await?;
        for set in bindings.iter().filter(|set| {
            !foreign.contains(&set.github_id)
                && !labels.iter().any(|label| label.id == set.label_id)
        }) {
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
            let name = retirement_name(&format!("ciproxy-retired-{}", set.id.simple()), labels);
            api.rename_scale_set(set.github_id, &name).await?;
            lease.check().await?;
        }
        Ok(foreign)
    }

    /// Isolate saved foreign ownership without deleting retained runners or work.
    async fn retire_foreign_bindings(
        &self,
        organization: &Organization,
        labels: &[Label],
        bindings: &[ScaleSet],
        api: &dyn ScaleSetApi,
        lease: &dyn MutationLease,
    ) -> Result<Vec<i64>> {
        let mut owners: Vec<_> = bindings
            .iter()
            .cloned()
            .map(|set| (organization.workspace_id, set))
            .collect();
        for registration in self.store.by_github_id(organization.github_id).await? {
            if registration.id != organization.id {
                owners.extend(
                    self.store
                        .scale_sets(&registration)
                        .await?
                        .into_iter()
                        .map(|set| (registration.workspace_id, set)),
                );
            }
        }
        owners.sort_by_key(|(_, set)| set.github_id);
        let mut retired = Vec::new();
        for (workspace, set) in owners
            .iter()
            .filter(|(workspace, _)| *workspace != organization.workspace_id)
        {
            if retired.contains(&set.github_id) {
                continue;
            }
            let shared = owners
                .iter()
                .any(|(owner, other)| owner != workspace && other.github_id == set.github_id);
            let base = if shared {
                quarantine_name(set.github_id)
            } else {
                format!("ciproxy-retired-{}", set.id.simple())
            };
            let name = retirement_name(&base, labels);
            lease.check().await?;
            api.rename_scale_set(set.github_id, &name).await?;
            lease.check().await?;
            retired.push(set.github_id);
        }
        Ok(retired)
    }
}
