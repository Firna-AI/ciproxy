//! Independent recovery of durable last-organization billing cancellations.

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_billing::service::api::BillingApi;
use ciproxy_store_interface::billing::store::BillingStore;

use crate::error::{ErrorResultExt, Result};

/// Background billing work, independent of GitHub membership maintenance.
#[async_trait]
pub trait BillingMaintenance: Send + Sync {
    /// Read durable cancellation requests and retry each workspace independently.
    async fn reconcile(&self) -> Result<()>;
}

/// Store-backed recovery using the same billing service as the dashboard.
pub struct BillingCancellations {
    /// Durable source of unfinished last-organization requests.
    pub store: Arc<dyn BillingStore>,
    /// Shared Stripe billing boundary.
    pub billing: Arc<dyn BillingApi>,
}

#[async_trait]
impl BillingMaintenance for BillingCancellations {
    async fn reconcile(&self) -> Result<()> {
        let store = self.store.clone();
        let workspaces = tokio::task::spawn_blocking(move || {
            let mut workspaces = store.billing_cancellations()?;
            workspaces.extend(store.billing_payment_workspaces()?);
            workspaces.sort();
            workspaces.dedup();
            ciproxy_store_interface::error::Result::Ok(workspaces)
        })
        .await
        .internal()?
        .internal()?;
        for workspace in workspaces {
            let billing = self.billing.clone();
            let result =
                tokio::task::spawn_blocking(move || billing.reconcile_removal(workspace)).await;
            match result {
                Ok(Ok(())) => {}
                Ok(Err(error)) => {
                    tracing::warn!(%workspace, %error, "billing cancellation will retry");
                }
                Err(error) => {
                    tracing::warn!(%workspace, %error, "billing cancellation worker will retry");
                }
            }
            let billing = self.billing.clone();
            let result =
                tokio::task::spawn_blocking(move || billing.reconcile_payments(workspace)).await;
            match result {
                Ok(Ok(())) => {}
                Ok(Err(error)) => {
                    tracing::warn!(%workspace,%error,"billing payment recovery will retry")
                }
                Err(error) => tracing::warn!(%workspace,%error,"billing payment worker will retry"),
            }
        }
        Ok(())
    }
}
