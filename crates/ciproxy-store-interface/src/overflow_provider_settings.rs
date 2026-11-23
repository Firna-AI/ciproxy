//! Workspace configuration that persists across overflow key versions.

use uuid::Uuid;

use crate::error::Result;
use crate::overflow::provider::OverflowProvider;

/// Saved settings for one provider in a workspace.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OverflowProviderSettings {
    /// Positive monthly spend limit in millionths of a US dollar, if set.
    pub spend_limit_usd_micros: Option<i64>,
    /// Whether the provider has been offered to the workspace's Linux labels.
    pub offered: bool,
}

/// Read and write the settings of one overflow provider at a time.
pub trait OverflowProviderSettingsStore: Send + Sync {
    /// Read saved settings, if the workspace has a record for this provider.
    fn overflow_provider_settings(
        &self,
        workspace_id: Uuid,
        provider: OverflowProvider,
    ) -> Result<Option<OverflowProviderSettings>>;

    /// Save a positive limit. An absent limit is valid only until one is set.
    fn save_overflow_provider_spend_limit(
        &self,
        workspace_id: Uuid,
        provider: OverflowProvider,
        spend_limit_usd_micros: Option<i64>,
    ) -> Result<()>;

    /// Record the first offer and append the provider to each active Linux label.
    /// Later offers leave the saved order and limit as they are.
    fn offer_overflow_provider(&self, workspace_id: Uuid, provider: OverflowProvider)
    -> Result<()>;
}
