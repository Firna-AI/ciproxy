//! Tenant configuration for routing and repository selection.

use uuid::Uuid;

use crate::error::Result;
use crate::overflow::provider::OverflowProvider;

/// The allowed workspace overflow waits.
#[derive(Clone, Copy, Debug, Eq, PartialEq, strum::EnumString, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum OverflowWait {
    /// Start overflow at once.
    AtOnce,
    /// Wait 15 seconds.
    Seconds15,
    /// Wait 30 seconds.
    Seconds30,
    /// Wait 60 seconds.
    Seconds60,
    /// Wait 120 seconds.
    Seconds120,
}

/// Routing settings of one workspace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Settings {
    /// Whether the workspace permits new overflow runners.
    pub overflow_enabled: bool,
    /// How long an eligible job waits for an own machine.
    pub overflow_wait: OverflowWait,
}

/// Read and replace tenant routing settings.
pub trait SettingsStore: Send + Sync {
    /// Read the saved settings of one workspace.
    fn settings(&self, workspace_id: Uuid) -> Result<Settings>;
    /// Replace the workspace switch and wait together.
    fn save_settings(&self, workspace_id: Uuid, settings: &Settings) -> Result<()>;
    /// Replace the workspace switch, wait, and one provider's limit together.
    fn save_settings_with_provider_limit(
        &self,
        workspace_id: Uuid,
        settings: &Settings,
        provider: OverflowProvider,
        spend_limit_usd_micros: i64,
    ) -> Result<()>;
}
