//! Reconciliation reads saved Boat supply and dispatches independent remote work.

use ciproxy_overflow::view::OverflowView;
use ciproxy_scaler::snapshot::Snapshot;
use ciproxy_store_interface::overflow::record::StartPhase;
use ciproxy_store_interface::own_runners::RoutingSnapshot;
use uuid::Uuid;

use crate::boat_snapshot;
use crate::error::{ErrorResultExt, Result};
use crate::runtime::MachineRuntime;
use crate::runtime_tasks::Work;

impl MachineRuntime {
    pub(crate) async fn reconcile_boat(
        &self,
        input: &RoutingSnapshot,
        snapshot: &mut Snapshot,
        refresh: bool,
    ) {
        let Some(overflow) = &self.overflow else {
            snapshot.sandboxes.clear();
            return;
        };
        let workspace = input.workspace_id;
        if let Ok(view) = OverflowView::from_snapshot(input.overflow.clone(), self.clock.now()) {
            match if refresh {
                view.retained_scaler_account()
            } else {
                view.scaler_account(chrono::DateTime::UNIX_EPOCH)
            } {
                Ok(facts) => snapshot
                    .providers
                    .push(boat_snapshot::account(snapshot.root_sizing, facts)),
                Err(error) => tracing::warn!(%error,"Boat accounting prevents new starts"),
            }
        }
        if refresh {
            if let Err(error) = self.cleanup_boat(workspace).await {
                tracing::warn!(%error,"Boat cleanup dispatch will retry");
            }
            let overflow = overflow.clone();
            let runtime = self.worker();
            if let Err(error) = self.dispatch(Work::BoatRefresh(workspace), async move {
                runtime
                    .task_scope()?
                    .wait_for(Work::BoatCleanup(workspace))
                    .await;
                {
                    let boat = runtime.boat_operation(workspace).await;
                    let _boat = boat.lock().await;
                    overflow.view(workspace).await.internal()?;
                }
                let _operation = runtime.operation.lock().await;
                runtime.reconcile_workspace_saved(workspace).await
            }) {
                tracing::warn!(%error,"Boat refresh dispatch will retry");
            }
        }
        for intent in &input.overflow_intents {
            if self
                .task_scope()
                .is_ok_and(|tasks| tasks.contains(Work::Create(intent.runner_id)))
            {
                continue;
            }
            let record = input
                .overflow
                .starts
                .iter()
                .find(|record| record.delivery_id == intent.delivery_id);
            if record.is_none_or(|record| record.ends_at.is_some()) {
                if let Err(error) = self
                    .retire_boat_registration(workspace, intent, record)
                    .await
                {
                    tracing::warn!(%error,"Boat registration cleanup dispatch will retry");
                }
            } else if let Some(record) = record
                && !matches!(record.phase, StartPhase::Running | StartPhase::Deleting)
            {
                let delivery = record.delivery_id;
                let overflow = overflow.clone();
                let runtime = self.worker();
                if let Err(error) = self.dispatch(Work::BoatRecovery(delivery), async move {
                    let boat = runtime.boat_operation(workspace).await;
                    let _boat = boat.lock().await;
                    overflow.resume(workspace, delivery).await.internal()?;
                    Ok(())
                }) {
                    tracing::warn!(%error,"Boat delivery recovery dispatch will retry");
                }
            }
        }
    }

    pub(crate) async fn cleanup_boat(&self, workspace: Uuid) -> Result<()> {
        if let Some(overflow) = &self.overflow {
            let overflow = overflow.clone();
            let runtime = self.worker();
            self.dispatch(Work::BoatCleanup(workspace), async move {
                let boat = runtime.boat_operation(workspace).await;
                let _boat = boat.lock().await;
                overflow.cleanup(workspace).await.internal()
            })?;
        }
        Ok(())
    }
}
