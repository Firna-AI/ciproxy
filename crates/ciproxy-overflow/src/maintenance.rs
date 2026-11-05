//! Inspection and deletion capabilities that cannot dispatch or launch a guest.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sandbox_interface::{
    BackendCreateSandboxRequest, BackendManagedSandbox, BackendSandbox, ProviderRef,
    SandboxLifecycle,
};
use unimock::unimock;
use uuid::Uuid;

use crate::error::Result;
use crate::provider::{BoatAccountApi, RuntimeUsage};

/// Read-only evidence from the fixed launcher's durable marker.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LaunchState {
    /// The launch operation created its once-only marker.
    Started,
    /// The read confirmed that no launch marker exists.
    NotStarted,
}

/// Inspect guest launch evidence without invoking or preparing a guest.
#[unimock(api = GuestInspectionMock)]
#[async_trait]
pub trait GuestInspection: Send + Sync {
    /// Inspect the root-owned marker. An unavailable reading returns an error.
    async fn inspect_launch(&self, sandbox: ProviderRef, delivery_id: Uuid) -> Result<LaunchState>;
}

/// A retained key's capabilities without create, file transfer, or guest launch.
#[derive(Clone)]
pub struct MaintenanceCapabilities {
    lifecycle: Arc<dyn SandboxLifecycle>,
    account: Arc<dyn BoatAccountApi>,
    inspection: Arc<dyn GuestInspection>,
}

impl MaintenanceCapabilities {
    /// Restrict the provider objects at the composition boundary.
    pub fn new(
        lifecycle: Arc<dyn SandboxLifecycle>,
        account: Arc<dyn BoatAccountApi>,
        inspection: Arc<dyn GuestInspection>,
    ) -> Self {
        Self {
            lifecycle,
            account,
            inspection,
        }
    }

    /// Recover a prior dispatch using reads only.
    pub async fn recover_sandbox_create(
        &self,
        request: BackendCreateSandboxRequest,
    ) -> sandbox_interface::Result<Option<BackendSandbox>> {
        self.lifecycle.recover_sandbox_create(request).await
    }

    /// Read the current state of a known sandbox.
    pub async fn inspect_sandbox(
        &self,
        sandbox: ProviderRef,
    ) -> sandbox_interface::Result<BackendSandbox> {
        self.lifecycle.inspect_sandbox(sandbox).await
    }

    /// Read marked inventory without changing any sandbox.
    pub async fn list_managed_sandboxes(
        &self,
        deployment: String,
    ) -> sandbox_interface::Result<Vec<BackendManagedSandbox>> {
        self.lifecycle.list_managed_sandboxes(deployment).await
    }

    /// Delete one proven owned sandbox.
    pub async fn destroy_sandbox(&self, sandbox: ProviderRef) -> sandbox_interface::Result<()> {
        self.lifecycle.destroy_sandbox(sandbox).await
    }

    /// Read one owned sandbox's measured runtime.
    pub async fn usage(
        &self,
        sandbox: ProviderRef,
        since: Option<DateTime<Utc>>,
        until: DateTime<Utc>,
    ) -> Result<Option<RuntimeUsage>> {
        self.account.usage(sandbox, since, until).await
    }

    /// Read durable launch evidence without replaying the launch operation.
    pub async fn inspect_launch(
        &self,
        sandbox: ProviderRef,
        delivery: Uuid,
    ) -> Result<LaunchState> {
        self.inspection.inspect_launch(sandbox, delivery).await
    }
}
