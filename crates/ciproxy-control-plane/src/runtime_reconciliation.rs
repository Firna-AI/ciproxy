//! Short workspace reservations dispatch remote work through the task supervisor.

use std::future::Future;
use std::pin::Pin;

use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_github::persistence::lease::LeaseStore;
use ciproxy_scaler::actions::{Action, Destination};
use uuid::Uuid;

use crate::error::{Error, ErrorResultExt, Result};
use crate::runtime::MachineRuntime;
use crate::snapshot::build;

impl MachineRuntime {
    /// Run ordinary best-effort routing under this workspace execution lease.
    pub(crate) async fn reconcile_workspace(&self, workspace_id: Uuid) -> Result<()> {
        self.reconcile_workspace_inner(workspace_id, true).await
    }

    pub(crate) fn reconcile_workspace_saved(
        &self,
        workspace_id: Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            let Some(lease) = self.store.own_runner_lease(workspace_id).await.internal()? else {
                return Ok(());
            };
            lease.check().await.internal()?;
            let input = self.load(workspace_id).await?;
            let runnable = self.runnable_organizations(&input).await?;
            let sizing = self
                .image
                .as_ref()
                .map_or_else(RootSizing::default, |image| RootSizing {
                    image_root_bytes: image.root_disk_bytes,
                });
            let mut snapshot = build(&input, &runnable, self.clock.now(), sizing);
            self.reconcile_boat(&input, &mut snapshot, false).await;
            for action in ciproxy_scaler::decision::decide(&snapshot) {
                let boat = match &action {
                    Action::StartRunner(start) => matches!(start.destination, Destination::Boat(_)),
                    Action::DeleteSandbox { .. } => true,
                    Action::StopRunner { runner_id, .. } => input
                        .boat_intents
                        .iter()
                        .any(|intent| intent.runner_id.to_string() == runner_id.0),
                };
                if boat {
                    self.apply(&input, action).await?;
                }
            }
            Ok(())
        })
    }

    async fn reconcile_workspace_inner(&self, workspace_id: Uuid, refresh: bool) -> Result<()> {
        let Some(lease) = self.store.own_runner_lease(workspace_id).await.internal()? else {
            return Ok(());
        };
        lease.check().await.internal()?;
        if let Err(error) = self.purge_boat_registration_metadata(workspace_id).await {
            tracing::warn!(%workspace_id,%error,"Boat registration metadata cleanup will retry");
        }
        let mut input = self.load(workspace_id).await?;
        self.reconcile_reports(&input).await?;
        input = self.load(workspace_id).await?;
        self.recover_stops(&input, &*lease).await?;
        for job in &input.jobs {
            self.route_boat_job(&input, job).await?;
            self.route_job(&input, job).await?;
        }
        self.ensure_image(&input).await?;
        for machine in &input.machines {
            if let Err(error) = self.send_docker_budget(&input, machine.machine.id).await {
                tracing::warn!(machine=%machine.machine.id,%error,"Docker policy update will retry");
            }
        }
        let root_sizing = self
            .image
            .as_ref()
            .map_or_else(RootSizing::default, |image| RootSizing {
                image_root_bytes: image.root_disk_bytes,
            });
        let runnable = self.runnable_organizations(&input).await?;
        let mut snapshot = build(&input, &runnable, self.clock.now(), root_sizing);
        self.reconcile_boat(&input, &mut snapshot, refresh).await;
        for action in ciproxy_scaler::decision::decide(&snapshot) {
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
}
