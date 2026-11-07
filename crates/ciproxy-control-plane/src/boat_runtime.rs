//! Wire pure Boat actions to the existing durable overflow executor.

use chrono::DateTime;
use ciproxy_github::persistence::session::ListenerTarget;
use ciproxy_overflow::api::{StartOutcome, StartRequest};
use ciproxy_overflow::view::OverflowView;
use ciproxy_protocol::identifiers::OrganizationId;
use ciproxy_protocol::secret::Secret;
use ciproxy_scaler::actions::StartRunner;
use ciproxy_scaler::snapshot::Snapshot;
use ciproxy_store_interface::boat_runtime::BoatRunnerIntent;
use ciproxy_store_interface::overflow::record::{StartPhase, StartRecord};
use ciproxy_store_interface::own_runners::RoutingSnapshot;
use secrecy::ExposeSecret;
use uuid::Uuid;

use crate::boat_snapshot;
use crate::error::{Error, ErrorResultExt, Result};
use crate::runtime::MachineRuntime;

impl MachineRuntime {
    pub(crate) async fn reconcile_boat(&self, input: &RoutingSnapshot, snapshot: &mut Snapshot) {
        let Some(overflow) = &self.overflow else {
            return;
        };
        if let Err(error) = self.cleanup_boat(input.workspace_id).await {
            tracing::warn!(%error, "Boat cleanup will retry");
        }
        let workspace = input.workspace_id;
        let current = match self.load(workspace).await {
            Ok(current) => current,
            Err(error) => {
                tracing::warn!(%error,"Boat registration recovery will retry");
                return;
            }
        };
        let intents = current.boat_intents.clone();
        let view = match OverflowView::from_snapshot(current.overflow.clone(), self.clock.now()) {
            Ok(view) => view,
            Err(error) => {
                tracing::warn!(%error,"Boat durable supply read will retry");
                return;
            }
        };
        let runnable = match self.runnable_organizations(&current).await {
            Ok(runnable) => runnable,
            Err(error) => {
                tracing::warn!(%error,"Boat admission read will retry");
                return;
            }
        };
        *snapshot =
            crate::snapshot::build(&current, &runnable, self.clock.now(), snapshot.root_sizing);
        for intent in &intents {
            let record = view
                .starts
                .iter()
                .find(|record| record.delivery_id == intent.delivery_id);
            if record.is_none_or(|record| record.ends_at.is_some()) {
                if let Err(error) = self
                    .retire_boat_registration(workspace, intent, record)
                    .await
                {
                    tracing::warn!(%error,"Boat registration cleanup will retry");
                }
            } else if let Some(record) = record {
                if !matches!(record.phase, StartPhase::Running | StartPhase::Deleting)
                    && let Err(error) = overflow.resume(workspace, record.delivery_id).await
                {
                    tracing::warn!(%error, "Boat delivery recovery will retry");
                }
                if let Some(id) = intent.github_id {
                    let store = self.store.clone();
                    let runner = intent.runner_id;
                    if !matches!(
                        tokio::task::spawn_blocking(
                            move || store.register_boat_runner(workspace, runner, id)
                        )
                        .await,
                        Ok(Ok(()))
                    ) {
                        tracing::warn!("Boat registration identity update will retry");
                    }
                }
            }
        }
        let (view, refreshed) = match overflow.view(workspace).await {
            Ok(view) => (view, true),
            Err(error) => {
                tracing::warn!(%error,"Boat is unavailable; durable supply remains reserved");
                (view, false)
            }
        };
        snapshot.boat = view
            .retained_scaler_account()
            .unwrap_or_else(|_| snapshot.boat.clone());
        if refreshed {
            match view.scaler_account(DateTime::UNIX_EPOCH) {
                Ok(account) => snapshot.boat = account,
                Err(error) => tracing::warn!(%error, "Boat accounting prevents new starts"),
            }
        }
        boat_snapshot::append(
            snapshot,
            &intents,
            &view.starts,
            &current.jobs,
            &current.labels,
        );
    }

    pub(crate) async fn cleanup_boat(&self, workspace: Uuid) -> Result<()> {
        if let Some(overflow) = &self.overflow {
            overflow.cleanup(workspace).await.internal()?;
        }
        Ok(())
    }

    pub(crate) async fn start_boat(
        &self,
        input: &RoutingSnapshot,
        mut action: StartRunner,
    ) -> Result<()> {
        let overflow = self.overflow.as_ref().ok_or(Error::StateChanged)?;
        let organization: Uuid = action.organization_id.0.parse().internal()?;
        let github_organization = input
            .organizations
            .iter()
            .find(|candidate| candidate.id == organization)
            .ok_or(Error::StateChanged)?
            .github_id;
        action.organization_id = OrganizationId(github_organization.to_string());
        let label = input
            .labels
            .iter()
            .find(|label| label.name == action.label.0)
            .ok_or(Error::StateChanged)?;
        let workspace = input.workspace_id;
        let store = self.store.clone();
        let sets = tokio::task::spawn_blocking(move || store.scale_sets(workspace, organization))
            .await
            .internal()?
            .internal()?;
        let set = sets
            .iter()
            .find(|set| set.label_id == label.id)
            .ok_or(Error::StateChanged)?;
        let intent = BoatRunnerIntent {
            delivery_id: self.clock.uuid(),
            runner_id: self.clock.uuid(),
            organization_id: organization,
            label_id: label.id,
            label_name: label.name.clone(),
            scale_set_id: set.id,
            requested_size: action.requested_size,
            size: action.size,
            required_root_disk_bytes: self
                .image
                .as_ref()
                .map_or_else(ciproxy_agent::root_sizing::RootSizing::default, |image| {
                    ciproxy_agent::root_sizing::RootSizing {
                        image_root_bytes: image.root_disk_bytes,
                    }
                })
                .root_bytes(action.requested_size.disk_bytes)
                .ok_or(Error::StateChanged)?,
            github_id: None,
            created_at: self.clock.now(),
        };
        let saved = intent.clone();
        let store = self.store.clone();
        tokio::task::spawn_blocking(move || store.create_boat_runner_intent(workspace, &saved))
            .await
            .internal()?
            .internal()?;
        let registration = self
            .registrations
            .create(
                ListenerTarget {
                    workspace_id: workspace,
                    organization_id: organization,
                    scale_set_id: set.id,
                },
                &format!("ciproxy-{}", intent.runner_id),
            )
            .await
            .internal()?;
        let store = self.store.clone();
        let runner = intent.runner_id;
        let github_id = registration.runner_id;
        tokio::task::spawn_blocking(move || {
            store.register_boat_runner(workspace, runner, github_id)
        })
        .await
        .internal()?
        .internal()?;
        let outcome = overflow
            .start(
                workspace,
                StartRequest {
                    delivery_id: intent.delivery_id,
                    runner_id: intent.runner_id,
                    organization_id: organization,
                    label_id: label.id,
                    scale_set_id: set.id,
                    action,
                    registration: Secret::new(registration.configuration.expose_secret()),
                },
            )
            .await
            .internal()?;
        if let StartOutcome::Pending(record) | StartOutcome::Started(record) = outcome {
            self.boat_registration(workspace, &record, github_id)
                .await?;
        }
        Ok(())
    }

    async fn boat_registration(
        &self,
        workspace: Uuid,
        record: &StartRecord,
        id: i64,
    ) -> Result<()> {
        let store = self.store.clone();
        let runner = record.runner_id;
        tokio::task::spawn_blocking(move || store.register_boat_runner(workspace, runner, id))
            .await
            .internal()?
            .internal()
    }
}
