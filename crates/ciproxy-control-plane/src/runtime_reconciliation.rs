//! Short workspace reservations dispatch remote work through the task supervisor.

use std::collections::HashSet;

use ciproxy_github::persistence::lease::LeaseStore;
use ciproxy_scaler::actions::{Action, Destination, StopReason};
use ciproxy_store_interface::labels::{Label, LabelPlace};
use ciproxy_store_interface::own_runners::RoutingSnapshot;
use uuid::Uuid;

use crate::docker_budget::policy;
use crate::error::{Error, ErrorResultExt, Result};
use crate::overflow_snapshot::ProviderChecks;
use crate::runtime::MachineRuntime;
use crate::snapshot::build;

impl MachineRuntime {
    pub(crate) async fn reconcile_workspace_saved(
        &self,
        workspace_id: Uuid,
        checks: ProviderChecks<'_>,
        deferred: &HashSet<String>,
    ) -> Result<()> {
        let Some(lease) = self.store.own_runner_lease(workspace_id).await.internal()? else {
            return Ok(());
        };
        lease.check().await.internal()?;
        let input = self.load(workspace_id).await?;
        let runnable = self.runnable_organizations(&input).await?;
        let sizing = self.root_sizing();
        let mut snapshot = build(&input, &runnable, self.clock.now(), sizing);
        self.reconcile_overflow(&input, &mut snapshot, checks).await;
        for action in ciproxy_scaler::decision::decide(&snapshot) {
            let apply = if let Some(label_id) = machine_action_label(&action) {
                deferred.contains(label_id)
            } else if let Action::StopRunner { runner_id, .. } = &action {
                input
                    .overflow_intents
                    .iter()
                    .any(|intent| intent.runner_id.to_string() == runner_id.0)
            } else {
                true
            };
            if apply {
                lease.check().await.internal()?;
                self.apply(&input, action).await?;
            }
        }
        Ok(())
    }

    /// Run ordinary best-effort routing under this workspace execution lease.
    pub(crate) async fn reconcile_workspace(&self, workspace_id: Uuid) -> Result<()> {
        let Some(lease) = self.store.own_runner_lease(workspace_id).await.internal()? else {
            return Ok(());
        };
        lease.check().await.internal()?;
        if let Err(error) = self
            .purge_overflow_registration_metadata(workspace_id)
            .await
        {
            tracing::warn!(%workspace_id, %error, "Overflow registration metadata cleanup will retry");
        }
        let mut input = self.load(workspace_id).await?;
        self.reconcile_reports(&input).await?;
        input = self.load(workspace_id).await?;
        self.recover_stops(&input, &*lease).await?;
        for job in &input.jobs {
            self.route_overflow_job(&input, job).await?;
            self.route_job(&input, job).await?;
        }
        self.ensure_image(&input).await?;
        for machine in &input.machines {
            if let Err(error) = self
                .send_docker_budget(
                    input.workspace_id,
                    machine.machine.id,
                    policy(&input, machine.machine.id),
                )
                .await
            {
                tracing::warn!(machine=%machine.machine.id,%error,"Docker policy update will retry");
            }
        }
        let root_sizing = self.root_sizing();
        let runnable = self.runnable_organizations(&input).await?;
        let mut snapshot = build(&input, &runnable, self.clock.now(), root_sizing);
        let deferred: HashSet<_> = input
            .labels
            .iter()
            .filter(|label| self.provider_precedes_machine(&input, label))
            .map(|label| label.id.to_string())
            .collect();
        if !deferred.is_empty() {
            // Jobs waiting for a provider refresh cannot reserve machine capacity yet.
            snapshot
                .queue
                .sort_by_key(|job| deferred.contains(&job.target.label_id));
        }
        self.refresh_overflow(workspace_id, deferred.clone()).await;
        self.reconcile_overflow(&input, &mut snapshot, ProviderChecks::InProgress)
            .await;
        for action in ciproxy_scaler::decision::decide(&snapshot) {
            if machine_action_label(&action).is_some_and(|label_id| deferred.contains(label_id)) {
                continue;
            }
            lease.check().await.internal()?;
            match self.apply(&input, action).await {
                Ok(()) => {}
                Err(Error::StateChanged) => return Err(Error::StateChanged),
                Err(error) => {
                    tracing::warn!(%workspace_id,%error,"runner action will retry; independent actions continue")
                }
            }
        }
        Ok(())
    }

    fn provider_precedes_machine(&self, input: &RoutingSnapshot, label: &Label) -> bool {
        self.overflow.is_some()
            && input.settings.overflow_enabled
            && label.order.contains(&LabelPlace::Machine)
            && label
                .order
                .iter()
                .take_while(|place| **place != LabelPlace::Machine)
                .any(|place| match *place {
                    LabelPlace::Provider(provider) => {
                        self.providers.start_image(provider).is_ok()
                            && input.overflow.active_account(provider).is_some()
                    }
                    LabelPlace::Machine => false,
                })
    }
}

fn machine_action_label(action: &Action) -> Option<&str> {
    match action {
        Action::StartRunner(start) | Action::ReplaceUnsentStart { start, .. }
            if matches!(start.destination, Destination::Machine(_)) =>
        {
            Some(&start.target.label_id)
        }
        Action::StopRunner {
            reason: StopReason::WarmPreemption { label_id },
            ..
        } => Some(label_id),
        _ => None,
    }
}
