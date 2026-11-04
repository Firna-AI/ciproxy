//! Object-safe store composition and connection factories.

use std::sync::Arc;

use crate::boat::BoatStore;
use crate::error::Result;
use crate::gateway::GatewayStore;
use crate::github::GithubStore;
use crate::jobs::JobStore;
use crate::labels::LabelStore;
use crate::machines::MachineStore;
use crate::organizations::OrganizationStore;
use crate::runners::RunnerStore;
use crate::scale_sets::ScaleSetStore;
use crate::settings::SettingsStore;
use crate::tokens::TokenStore;
use crate::workspaces::WorkspaceStore;

/// The complete hosted persistence boundary.
pub trait Store:
    WorkspaceStore
    + OrganizationStore
    + LabelStore
    + ScaleSetStore
    + MachineStore
    + TokenStore
    + RunnerStore
    + JobStore
    + SettingsStore
    + BoatStore
    + GatewayStore
    + GithubStore
{
}

/// A composition-root adapter that opens a store and applies its migrations.
pub trait StoreFactory: Send + Sync {
    /// Connect to a database and apply pending forward migrations.
    fn connect(&self, database_url: &str) -> Result<Arc<dyn Store>>;
}
