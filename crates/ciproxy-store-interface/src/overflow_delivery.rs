//! Delivery authority excludes provider usage and billing measurements.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::overflow::provider::OverflowProvider;
use crate::overflow::record::{ProviderParameters, StartPhase, StartRecord};

/// Immutable delivery inputs and current lifecycle used by guarded remote results.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OverflowDeliveryGuard {
    /// The provider of the saved delivery.
    pub provider: OverflowProvider,
    /// Stable delivery identity.
    pub delivery_id: Uuid,
    /// Runner that owns this delivery.
    pub runner_id: Uuid,
    /// Organization that owns the registration.
    pub organization_id: Uuid,
    /// Original label identity.
    pub label_id: Uuid,
    /// Internal registration binding.
    pub scale_set_id: Uuid,
    /// Original virtual CPU request.
    pub requested_vcpus: u32,
    /// Original memory request in decimal GB.
    pub requested_memory_gb: u32,
    /// Original free job disk request in decimal GB.
    pub requested_disk_gb: u32,
    /// Exact released image root length.
    pub image_root_bytes: u64,
    /// Complete root allocation of the delivery.
    pub required_root_disk_bytes: u64,
    /// Own-machine fit at admission.
    pub own_machine_fits: bool,
    /// Fixed provider dispatch parameters.
    pub dispatch: Option<ProviderParameters>,
    /// Retained key identity.
    pub key_id: Uuid,
    /// Deployment correlation mark.
    pub deployment: String,
    /// Exact saved image snapshot.
    pub snapshot: String,
    /// Current startup or deletion phase.
    pub phase: StartPhase,
    /// Proven sandbox identity, independent of its measured lifetime bound.
    pub sandbox_id: Option<String>,
    /// Confirmed lifecycle end.
    pub ends_at: Option<DateTime<Utc>>,
    /// First accepted guest end report.
    pub reported_at: Option<DateTime<Utc>>,
}

impl From<StartRecord> for OverflowDeliveryGuard {
    fn from(record: StartRecord) -> Self {
        let StartRecord {
            provider,
            delivery_id,
            runner_id,
            organization_id,
            label_id,
            scale_set_id,
            requested_vcpus,
            requested_memory_gb,
            requested_disk_gb,
            image_root_bytes,
            required_root_disk_bytes,
            own_machine_fits,
            dispatch,
            key_id,
            deployment,
            snapshot,
            phase,
            sandbox,
            starts_at: _,
            launched_at: _,
            ends_at,
            billing: _,
            start_fee_bound_usd: _,
            start_usd: _,
            reported_at,
            runtime: _,
            runtime_total: _,
            running_observed_at: _,
            runtime_complete: _,
        } = record;
        Self {
            provider,
            delivery_id,
            runner_id,
            organization_id,
            label_id,
            scale_set_id,
            requested_vcpus,
            requested_memory_gb,
            requested_disk_gb,
            image_root_bytes,
            required_root_disk_bytes,
            own_machine_fits,
            dispatch,
            key_id,
            deployment,
            snapshot,
            phase,
            sandbox_id: sandbox.map(|sandbox| sandbox.id().to_owned()),
            ends_at,
            reported_at,
        }
    }
}
