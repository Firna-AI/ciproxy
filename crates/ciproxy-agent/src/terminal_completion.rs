//! Entry-specific local work required before the Controller forgets identity.

use std::sync::Arc;

use crate::boundaries::{Images, Service};
use crate::host_preparation::Preparation;

/// Terminal completion either stops the daemon or removes the installation.
pub enum CompletionAction {
    /// Preserve the installation and durable Removed mode without its identity.
    Stop,
    /// Remove local installation resources before clearing the saved state.
    Uninstall(UninstallCleanup),
}

/// Local uninstall dependencies, kept until checked terminal cleanup succeeds.
pub struct UninstallCleanup {
    /// Only a recovered, drained runtime owns image mutation.
    pub(crate) images: Option<Arc<dyn Images>>,
    /// Remove the owned area and runtime tools.
    pub(crate) preparation: Arc<dyn Preparation>,
    /// Remove the installed agent service.
    pub(crate) service: Arc<dyn Service>,
}
