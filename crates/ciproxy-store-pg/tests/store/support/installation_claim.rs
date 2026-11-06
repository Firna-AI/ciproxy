//! Linked token-free installation fixtures for existing retention scenarios.

use ciproxy_store_interface::installation_links::{InstallationAttempt, InstallationClaim};
use ciproxy_store_interface::organizations::Organization;
use ciproxy_store_interface::store::Store;
use uuid::Uuid;

use super::at;

pub(in crate::store) fn linked(store: &dyn Store, organization: &Organization) -> (Uuid, Vec<u8>) {
    let id = Uuid::new_v4();
    let digest = id.as_bytes().to_vec();
    let attempt = InstallationAttempt {
        id: Uuid::new_v4(),
        state_digest: digest.clone(),
        requester_id: 7,
        direct: false,
        installation_id: None,
    };
    store.begin_installation_attempt(&attempt).unwrap();
    let claim = InstallationClaim {
        id,
        installation_id: organization.installation_id,
        github_id: organization.github_id,
        login: organization.login.clone(),
        requester_id: 7,
        event_seen: true,
        user_verified: true,
        direct: false,
        workspace_id: Some(organization.workspace_id),
        organization_id: Some(organization.id),
        verified_at: Some(at()),
    };
    store
        .save_installation_claim(&claim, Some(attempt.id))
        .unwrap();
    store
        .offer_installation_workspaces(id, 7, &[organization.workspace_id])
        .unwrap();
    (id, digest)
}
