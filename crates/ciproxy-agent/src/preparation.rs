//! Host preparation and uninstall ports that each platform implements.

use async_trait::async_trait;

use crate::error::{Error, Result, repair};

/// Prepare, verify, and remove the host parts that job VMs need.
#[cfg_attr(test, unimock::unimock(api = PreparationMock))]
#[async_trait]
pub(crate) trait Preparation: Send + Sync {
    async fn prepare(&self) -> Result<()>;
    async fn verify(&self) -> Result<()>;
    async fn remove(&self) -> Result<()>;
}

/// Remove the installation after its recorded VMs finish.
#[async_trait]
pub(crate) trait Uninstall: Send + Sync {
    /// Remove an empty machine or prepare its recorded VMs for draining.
    async fn start(&self) -> Result<UninstallState>;
    /// Remove the area and identity only after every recorded VM finishes.
    async fn finish(&self) -> Result<()>;
}

/// Cleanup completes immediately without VMs or waits for their drain loop.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UninstallState {
    /// Guarded cleanup removed the empty machine.
    Complete,
    /// The caller must drain recorded VMs and then finish cleanup.
    Draining,
}

/// Keep a typed repair error, or log the cause and name the repair.
pub(crate) fn stage_error(error: Error, component: &'static str, action: &'static str) -> Error {
    match error {
        error @ Error::Preparation { .. } => error,
        error => {
            tracing::warn!(%error, component, "host preparation needs repair");
            repair(component, action)
        }
    }
}
