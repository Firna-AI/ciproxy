//! Ordered immutable lifecycle delivery, confirmed only after consumer acceptance.

use ciproxy_store_interface::github_lifecycle::LifecycleEvent;

use crate::error::Result;
use crate::persistence::mutation::MutationLease;

use super::AppService;

impl AppService {
    pub(super) async fn replay_pending_lifecycle(&self, github_id: i64) -> Result<()> {
        let lease = self.leases.acquire(github_id).await?;
        self.deliver_lifecycle(github_id, &*lease).await
    }

    pub(super) async fn deliver_lifecycle(
        &self,
        github_id: i64,
        lease: &dyn MutationLease,
    ) -> Result<()> {
        lease.check().await?;
        for delivery in self.outbox.pending(github_id).await? {
            lease.check().await?;
            if !matches!(delivery.event, LifecycleEvent::Removed { .. }) {
                let workspace_id = delivery.event.organization().workspace_id;
                if let Some(user_id) = self.store.pending_admin(workspace_id).await? {
                    self.authority
                        .ensure_initial_admin(workspace_id, user_id)
                        .await?;
                    self.store.confirm_admin(workspace_id).await?;
                }
            }
            self.lifecycle.changed(delivery.event).await?;
            lease.check().await?;
            self.outbox.confirm(github_id, delivery.sequence).await?;
        }
        Ok(())
    }
}
