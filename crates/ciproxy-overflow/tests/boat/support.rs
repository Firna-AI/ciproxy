//! Literal marked create request for the pinned Boat adapter.

use sandbox_interface::{
    BackendCreateSandboxRequest, OperationId, ProviderRef, ResourceOwner, SandboxConsumer,
    SandboxId, SandboxLifetime, SandboxNetworkPolicy, SandboxSizeSelection,
};
use uuid::Uuid;

pub(super) fn request() -> BackendCreateSandboxRequest {
    BackendCreateSandboxRequest {
        sandbox_id: SandboxId::from_uuid(Uuid::from_u128(12)),
        operation_id: OperationId::from_uuid(Uuid::from_u128(11)),
        owner: ResourceOwner::platform(Uuid::from_u128(1)),
        consumer: SandboxConsumer::Runtime,
        lifetime: SandboxLifetime::OneShot {
            max_lifetime: std::time::Duration::from_secs(21600),
        },
        size: Some(SandboxSizeSelection::ProviderSize("large".to_owned())),
        deployment_id: "ciproxy-live-fixture/workspace-1".to_owned(),
        profile: "ciproxy".to_owned(),
        network: SandboxNetworkPolicy::Open,
        snapshot_provider_ref: Some(ProviderRef::new("shared-image-v7")),
    }
}
