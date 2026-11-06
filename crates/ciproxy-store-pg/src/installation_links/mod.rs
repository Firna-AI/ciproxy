//! Exact installation association and independent local activation queries.

pub(crate) mod activation;
mod association;
mod attempts;
mod claims;
mod offers;
pub(crate) mod retention;
mod transaction;

use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::installation_association::SignedInstallation;
use ciproxy_store_interface::installation_links::{
    InstallationAttempt, InstallationClaim, InstallationLinkStore, OrganizationActivation,
};
use uuid::Uuid;

use crate::connection::PgStore;

impl InstallationLinkStore for PgStore {
    fn associate_signed_installation(
        &self,
        signed: &SignedInstallation,
    ) -> Result<InstallationClaim> {
        association::signed(self, signed)
    }
    fn complete_personal_installation_attempt(
        &self,
        id: Uuid,
        requester: i64,
        installation: i64,
    ) -> Result<()> {
        attempts::complete_personal(self, id, requester, installation)
    }
    fn installation_attempt_by_id(&self, id: Uuid) -> Result<Option<InstallationAttempt>> {
        attempts::by_id(self, id)
    }
    fn has_add_installation(&self, requester: i64, installation: i64) -> Result<bool> {
        attempts::bound(self, requester, installation)
    }
    fn pending_installation_choices(&self) -> Result<Vec<Uuid>> {
        claims::pending_choices(self)
    }
    fn begin_installation_attempt(&self, attempt: &InstallationAttempt) -> Result<()> {
        attempts::save(self, attempt)
    }
    fn installation_attempt(&self, digest: &[u8]) -> Result<Option<InstallationAttempt>> {
        attempts::find(self, digest)
    }
    fn has_pending_add(&self, requester: i64) -> Result<bool> {
        attempts::pending(self, requester)
    }
    fn save_installation_claim(
        &self,
        claim: &InstallationClaim,
        attempt: Option<Uuid>,
    ) -> Result<InstallationClaim> {
        claims::save(self, claim, attempt)
    }
    fn installation_claim(&self, id: Uuid) -> Result<Option<InstallationClaim>> {
        claims::find(self, id)
    }
    fn claim_for_installation(&self, id: i64) -> Result<Option<InstallationClaim>> {
        claims::installation(self, id)
    }
    fn installation_claims(&self, requester: i64) -> Result<Vec<InstallationClaim>> {
        claims::owned(self, requester)
    }
    fn offer_installation_workspaces(
        &self,
        id: Uuid,
        requester: i64,
        workspaces: &[Uuid],
    ) -> Result<()> {
        offers::save(self, id, requester, workspaces)
    }
    fn installation_workspace_offers(&self, id: Uuid, requester: i64) -> Result<Vec<Uuid>> {
        offers::read(self, id, requester)
    }
    fn choose_installation_workspace(
        &self,
        id: Uuid,
        requester: i64,
        workspace: Uuid,
    ) -> Result<()> {
        offers::choose(self, id, requester, workspace)
    }
    fn authorize_installation_claim(&self, id: Uuid, approved: bool) -> Result<()> {
        offers::authorize(self, id, approved)
    }
    fn organization_activation(
        &self,
        workspace: Uuid,
        organization: Uuid,
    ) -> Result<OrganizationActivation> {
        activation::read(self, workspace, organization)
    }
    fn organization_runnable(&self, workspace: Uuid, organization: Uuid) -> Result<bool> {
        activation::runnable(self, workspace, organization)
    }
    fn reconcile_organization_billing(&self, workspace: Uuid) -> Result<()> {
        activation::billing(self, workspace)
    }
    fn approve_organization_activation(&self, workspace: Uuid, organization: Uuid) -> Result<()> {
        activation::approve(self, workspace, organization)
    }
    fn pending_organization_activations(&self) -> Result<Vec<Uuid>> {
        activation::pending(self)
    }
}
