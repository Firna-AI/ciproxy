//! Reconciliation reads saved provider accounts and dispatches remote work.

use std::collections::HashSet;

use ciproxy_overflow::error::Error;
use ciproxy_overflow::view::{OverflowView, RefreshStatus};
use ciproxy_scaler::snapshot::Snapshot;
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use ciproxy_store_interface::overflow::record::StartPhase;
use ciproxy_store_interface::own_runners::RoutingSnapshot;
use uuid::Uuid;

use crate::error::{ErrorResultExt, Result};
use crate::overflow_snapshot;
use crate::runtime::MachineRuntime;
use crate::runtime_tasks::Work;

impl MachineRuntime {
    pub(crate) async fn reconcile_overflow(
        &self,
        input: &RoutingSnapshot,
        snapshot: &mut Snapshot,
        refreshed: &[OverflowProvider],
    ) {
        let Some(overflow) = &self.overflow else {
            snapshot.sandboxes.clear();
            return;
        };
        let workspace = input.workspace_id;
        if let Ok(view) = OverflowView::from_snapshot(input.overflow.clone(), self.clock.now()) {
            for (provider, account) in
                overflow_snapshot::accounts(&view, &self.providers, refreshed)
            {
                match account {
                    Ok(account) => snapshot.providers.push(account),
                    Err(error) => {
                        tracing::warn!(provider = provider.as_str(), %error, "Provider facts prevent new starts")
                    }
                }
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
                    .retire_overflow_registration(workspace, intent, record)
                    .await
                {
                    tracing::warn!(%error, "Overflow registration cleanup dispatch will retry");
                }
            } else if let Some(record) = record
                && !matches!(record.phase, StartPhase::Running | StartPhase::Deleting)
            {
                let delivery = record.delivery_id;
                let overflow = overflow.clone();
                let runtime = self.worker();
                if let Err(error) = self.dispatch(Work::OverflowRecovery(delivery), async move {
                    let operation = runtime.overflow_operation(workspace).await;
                    let _operation = operation.lock().await;
                    overflow.resume(workspace, delivery).await.internal()?;
                    Ok(())
                }) {
                    tracing::warn!(%error, "Overflow delivery recovery dispatch will retry");
                }
            }
        }
    }

    pub(crate) async fn refresh_overflow(&self, workspace: Uuid, deferred: HashSet<String>) {
        let Some(overflow) = &self.overflow else {
            return;
        };
        if let Err(error) = self.cleanup_overflow(workspace).await {
            tracing::warn!(%error, "Overflow cleanup dispatch will retry");
        }
        let overflow = overflow.clone();
        let runtime = self.worker();
        if let Err(error) = self.dispatch(Work::OverflowRefresh(workspace), async move {
            runtime
                .task_scope()?
                .wait_for(Work::OverflowCleanup(workspace))
                .await;
            let view = {
                let operation = runtime.overflow_operation(workspace).await;
                let _operation = operation.lock().await;
                overflow.view(workspace).await
            };
            let refreshed: Vec<_> = view
                .iter()
                .flat_map(|view| &view.providers)
                .filter(|view| view.refresh == RefreshStatus::Current)
                .map(|view| view.provider)
                .collect();
            if !refreshed.is_empty() || !deferred.is_empty() {
                let _operation = runtime.operation.lock().await;
                runtime
                    .reconcile_workspace_saved(workspace, &refreshed, &deferred)
                    .await?;
            }
            let view = view.internal()?;
            if refreshed.is_empty()
                && view
                    .providers
                    .iter()
                    .any(|view| view.refresh == RefreshStatus::Failed)
            {
                return Err(Error::ProviderUnavailable).internal();
            }
            Ok(())
        }) {
            tracing::warn!(%error, "Overflow refresh dispatch will retry");
        }
    }

    pub(crate) async fn cleanup_overflow(&self, workspace: Uuid) -> Result<()> {
        if let Some(overflow) = &self.overflow {
            let overflow = overflow.clone();
            let runtime = self.worker();
            self.dispatch(Work::OverflowCleanup(workspace), async move {
                let operation = runtime.overflow_operation(workspace).await;
                let _operation = operation.lock().await;
                overflow.cleanup(workspace).await.internal()
            })?;
        }
        Ok(())
    }
}
