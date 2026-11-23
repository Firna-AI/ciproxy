//! Marked create request for the pinned Boat adapter.

use ciproxy_overflow::boat::BoatFactory;
use ciproxy_overflow::provider::{ProviderFactory, SandboxStart};
use sandbox_interface::BackendCreateSandboxRequest;
use uuid::Uuid;

pub(super) fn request() -> BackendCreateSandboxRequest {
    BoatFactory::new().create_request(SandboxStart {
        runner_id: Uuid::from_u128(12),
        delivery_id: Uuid::from_u128(11),
        workspace_id: Uuid::from_u128(1),
        lifetime_seconds: 21600,
        size: "large",
        deployment: "ciproxy-live-fixture/workspace-1",
        image: "shared-image-v7",
    })
}
