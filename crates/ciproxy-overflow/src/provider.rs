//! Shared sandbox capabilities and ciproxy's provider-specific seams.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ciproxy_protocol::secret::Secret;
use rust_decimal::Decimal;
use sandbox_interface::{
    BackendCreateSandboxRequest, BackendWriteFileRequest, ProviderRef, SandboxAccountCapacity,
    SandboxFiles, SandboxLifecycle,
};
use unimock::unimock;
use uuid::Uuid;

use crate::error::Result;
use crate::maintenance::{GuestInspection, MaintenanceCapabilities};

/// A verified billing account available to the submitted key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BillingAccount {
    /// Provider-specific account selector.
    pub id: String,
    /// Verified billing account name.
    pub name: String,
}

/// Accessible billing accounts and the provider's suggested selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyAccounts {
    /// Each billing account that the key can use.
    pub accounts: Vec<BillingAccount>,
    /// Identifier of the suggested billing account.
    pub selected_account: String,
}

/// Account facts that the shared capability intentionally does not model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountFacts {
    /// Verified account or organization display name.
    pub name: String,
    /// Provider-confirmed billing account selector.
    pub billing_account: String,
    /// Whether the provider reports a free trial.
    pub trial: bool,
    /// One-shot lifetime allowed for a sandbox of this account.
    pub lifetime_seconds: u32,
    /// Billing recovery is confirmed independently of temporary capacity.
    pub billing_ready: bool,
    /// Published upper bound on the start fee, when available.
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

/// Narrow provider reads for key verification, account facts, and runtime cost.
#[unimock(api = ProviderAccountApiMock)]
#[async_trait]
pub trait ProviderAccountApi: Send + Sync {
    /// Check the key and list its billing accounts without selecting one.
    async fn check_key(&self) -> Result<KeyAccounts>;
    /// Read facts for the selected billing account without reserving a sandbox.
    async fn account(&self) -> Result<AccountFacts>;
    /// Read only one sandbox's usage. Confirmed absence returns None.
    async fn usage(
        &self,
        sandbox: ProviderRef,
        since: Option<DateTime<Utc>>,
        until: DateTime<Utc>,
    ) -> Result<Option<RuntimeUsage>>;
}

/// Trusted operations that prepare and launch the guest in the provider image.
#[unimock(api = GuestLauncherMock)]
#[async_trait]
pub trait GuestLauncher: Send + Sync {
    /// Prepare a private directory and a mode-0600 registration target.
    async fn prepare(&self, sandbox: ProviderRef, delivery_id: Uuid) -> Result<()>;
    /// Invoke the fixed image entrypoint under a durable per-sandbox launch marker.
    async fn launch(&self, sandbox: ProviderRef, delivery_id: Uuid) -> Result<()>;
}

/// Explicit capability objects for one decrypted key and billing account.
#[derive(Clone)]
pub struct Capabilities {
    /// Shared lifecycle, including recover-only create.
    pub lifecycle: Arc<dyn SandboxLifecycle>,
    /// Shared eligible published sizes and hourly prices.
    pub capacity: Arc<dyn SandboxAccountCapacity>,
    /// Shared safe bounded registration-file transfer.
    pub files: Arc<dyn SandboxFiles>,
    /// Provider-specific account and runtime facts.
    pub account: Arc<dyn ProviderAccountApi>,
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

/// A provider's capability composition and request builders.
#[unimock(api = ProviderFactoryMock)]
pub trait ProviderFactory: Send + Sync {
    /// Construct account reads for a key check before choosing a billing account.
    fn for_key_check(&self, key: &Secret) -> Result<Arc<dyn ProviderAccountApi>>;
    /// Construct all capabilities for the same key and billing account.
    fn capabilities(&self, key: &Secret, billing_account: &str) -> Result<Capabilities>;
    /// Construct restricted capabilities for a retained key.
    fn maintenance(&self, key: &Secret, billing_account: &str) -> Result<MaintenanceCapabilities>;
    /// Build the same lifecycle request for the initial create and each recovery read.
    fn create_request(&self, start: SandboxStart<'_>) -> BackendCreateSandboxRequest;
    /// Give the file targets for the provider's guest preparation and launch.
    fn registration_files(&self, delivery_id: Uuid) -> RegistrationFiles;
}

/// Saved identities and fixed parameters of a single sandbox dispatch.
pub struct SandboxStart<'a> {
    /// Workspace that holds the sandbox.
    pub workspace_id: Uuid,
    /// Runner identity used as the sandbox identity.
    pub runner_id: Uuid,
    /// Delivery identity used as the idempotency key.
    pub delivery_id: Uuid,
    /// Inventory mark for the deployment and workspace.
    pub deployment: &'a str,
    /// Exact provider size.
    pub size: &'a str,
    /// Exact image reference saved at admission.
    pub image: &'a str,
    /// One-shot lifetime fixed at dispatch.
    pub lifetime_seconds: u32,
}

/// Files used by the provider's guest startup.
pub struct RegistrationFiles {
    /// Optional empty write required before preparing private input.
    pub bootstrap: Option<FileTarget>,
    /// The private registration target prepared by the guest launcher.
    pub registration: FileTarget,
}

/// A confined target for the shared file capability.
pub struct FileTarget {
    /// Root directory for a file write.
    pub root: String,
    /// Path relative to the root.
    pub path: String,
}

impl FileTarget {
    /// Build the bounded file write for this target.
    pub fn write_request(self, sandbox: ProviderRef, bytes: Vec<u8>) -> BackendWriteFileRequest {
        BackendWriteFileRequest {
            sandbox_provider_ref: sandbox,
            root: self.root,
            path: self.path,
            bytes,
        }
    }
}
