//! Supported sandbox capabilities and ciproxy's account and root-launch seams.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ciproxy_protocol::secret::Secret;
use rust_decimal::Decimal;
use sandbox_interface::{ProviderRef, SandboxAccountCapacity, SandboxFiles, SandboxLifecycle};
use unimock::unimock;
use uuid::Uuid;

use crate::error::Result;
use crate::maintenance::{GuestInspection, MaintenanceCapabilities};

/// Account facts that the shared capability intentionally does not model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountFacts {
    /// Verified account or organization display name.
    pub name: String,
    /// Provider-confirmed explicit billing wallet.
    pub wallet: String,
    /// Free-trial lifetime policy.
    pub trial: bool,
    /// Billing recovery is confirmed independently of temporary capacity.
    pub billing_ready: bool,
    /// Highest published start fee, with the next-start quote as a fallback.
    pub start_fee_bound_usd: Option<Decimal>,
}

/// A provider's measured sandbox runtime for one requested interval.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeUsage {
    /// First instant of the provider's measurement.
    pub since: DateTime<Utc>,
    /// Last instant measured by the provider.
    pub until: DateTime<Utc>,
    /// USD runtime cost, without the one-time start charge.
    pub usd: Decimal,
    /// Whether the provider is still counting runtime.
    pub running: bool,
}

/// Narrow Boat reads for account identity, trial lifetime, fees, and runtime.
#[unimock(api = BoatAccountApiMock)]
#[async_trait]
pub trait BoatAccountApi: Send + Sync {
    /// Validate the key and wallet, without reserving or starting a sandbox.
    async fn account(&self) -> Result<AccountFacts>;
    /// Read only one sandbox's usage. Confirmed absence returns None.
    async fn usage(
        &self,
        sandbox: ProviderRef,
        since: Option<DateTime<Utc>>,
        until: DateTime<Utc>,
    ) -> Result<Option<RuntimeUsage>>;
}

/// Trusted fixed-image operations that need Boat's root-capable controller.
#[unimock(api = GuestLauncherMock)]
#[async_trait]
pub trait GuestLauncher: Send + Sync {
    /// Prepare a private directory and a mode-0600 registration target.
    async fn prepare(&self, sandbox: ProviderRef, delivery_id: Uuid) -> Result<()>;
    /// Invoke the fixed image entrypoint under a durable per-sandbox launch marker.
    async fn launch(&self, sandbox: ProviderRef, delivery_id: Uuid) -> Result<()>;
}

/// Explicit capability objects for one decrypted key and fixed wallet.
#[derive(Clone)]
pub struct Capabilities {
    /// Shared lifecycle, including recover-only create.
    pub lifecycle: Arc<dyn SandboxLifecycle>,
    /// Shared eligible published sizes and hourly prices.
    pub capacity: Arc<dyn SandboxAccountCapacity>,
    /// Shared safe bounded registration-file transfer.
    pub files: Arc<dyn SandboxFiles>,
    /// Boat-only account and runtime facts.
    pub account: Arc<dyn BoatAccountApi>,
    /// Fixed privileged image invocation.
    pub launcher: Arc<dyn GuestLauncher>,
    /// Read-only evidence for lost launch acknowledgments.
    pub inspection: Arc<dyn GuestInspection>,
}

impl Capabilities {
    /// Expose only reads, recovery, and deletion to retained-key consumers.
    pub fn maintenance(&self) -> MaintenanceCapabilities {
        MaintenanceCapabilities::new(
            self.lifecycle.clone(),
            self.account.clone(),
            self.inspection.clone(),
        )
    }
}

/// Composition boundary that keeps the account key outside every workload.
#[unimock(api = BackendFactoryMock)]
pub trait BackendFactory: Send + Sync {
    /// Construct all capabilities with the same key, HTTPS origin, and wallet.
    fn backend(&self, key: &Secret, wallet: &str) -> Result<Capabilities>;
    /// Construct a restricted backend with no create, file, or launch authority.
    fn maintenance(&self, key: &Secret, wallet: &str) -> Result<MaintenanceCapabilities>;
}

/// Private input root shared by the file capability and fixed image launcher.
pub fn registration_root(delivery_id: Uuid) -> String {
    format!("/home/sandbox-workload/.ciproxy-{}", delivery_id.simple())
}
