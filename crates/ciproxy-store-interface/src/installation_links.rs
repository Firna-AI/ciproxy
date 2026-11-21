//! Durable installation association without retaining a person's credentials.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::Result;
use crate::installation_association::SignedInstallation;
use crate::installation_results::{InstallationResultDelivery, PendingInstallationResult};

/// One explicit browser entry into GitHub installation.
#[derive(Clone, Debug)]
pub struct InstallationAttempt {
    /// Exact completed return, absent while this entry still waits.
    pub installation_id: Option<i64>,
    /// Stable attempt identity.
    pub id: Uuid,
    /// Digest of the random state returned by GitHub.
    pub state_digest: Vec<u8>,
    /// Original person's GitHub identity.
    pub requester_id: i64,
    /// Explicit normal install, rather than Add organization.
    pub direct: bool,
}

/// Token-free association shared by the event and the verified original person.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InstallationClaim {
    /// Stable recovery identity.
    pub id: Uuid,
    /// Exact GitHub installation.
    pub installation_id: i64,
    /// Exact organization account.
    pub github_id: i64,
    /// Authoritatively observed account name.
    pub login: String,
    /// The signed requester's identity, never the approving owner's replacement.
    pub requester_id: i64,
    /// The signed creation event was observed.
    pub event_seen: bool,
    /// The original person proved access with their own fresh token.
    pub user_verified: bool,
    /// Explicit direct entry evidence, independent of missing state.
    pub direct: bool,
    /// Exact chosen workspace, absent before installed proof and choice.
    pub workspace_id: Option<Uuid>,
    /// Exact registration after linking.
    pub organization_id: Option<Uuid>,
    /// Trusted observation time of own-user proof.
    pub verified_at: Option<DateTime<Utc>>,
}

/// Recovery authority derived from the durable evidence for one installation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InstallationRecovery {
    /// Explicit Direct entry still needs its signed creation event.
    DirectAwaitingEvent,
    /// A signed Direct installation needs no Add proof or workspace choice.
    Direct,
    /// Add proof cannot finish before its signed creation event.
    AddAwaitingEvent,
    /// Add requires fresh installed proof from its original requester.
    AddAwaitingProof,
    /// Verified Add evidence still needs an exact workspace choice.
    AddAwaitingWorkspace,
    /// Signed Add evidence and original-person proof name one workspace.
    Add {
        /// The immutable destination selected from the exact offers.
        workspace_id: Uuid,
    },
}

impl InstallationClaim {
    /// Interpret Direct and Add proof requirements without changing stored facts.
    pub fn recovery(&self) -> InstallationRecovery {
        match (
            self.direct,
            self.event_seen,
            self.user_verified,
            self.workspace_id,
        ) {
            (true, false, _, _) => InstallationRecovery::DirectAwaitingEvent,
            (true, true, _, _) => InstallationRecovery::Direct,
            (false, false, _, _) => InstallationRecovery::AddAwaitingEvent,
            (false, true, false, _) => InstallationRecovery::AddAwaitingProof,
            (false, true, true, None) => InstallationRecovery::AddAwaitingWorkspace,
            (false, true, true, Some(workspace_id)) => InstallationRecovery::Add { workspace_id },
        }
    }
}

impl InstallationRecovery {
    /// Test whether stored evidence alone can retry exact installation work.
    pub fn is_ready(self) -> bool {
        matches!(self, Self::Direct | Self::Add { .. })
    }
}

/// Local execution authority, separate from GitHub suspension or removal.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct OrganizationActivation {
    /// Resources were prepared and initial admission completed.
    pub admitted: bool,
    /// The linked count exceeds the current tier.
    pub billing_hold: bool,
    /// Current Admin approval was absent at link time.
    pub admin_approval_hold: bool,
    /// Original requester, absent for registrations predating Add.
    pub added_by: Option<i64>,
}

impl OrganizationActivation {
    /// Preserve admitted authority, or require both initial holds to be clear.
    pub fn permits_activation(self) -> bool {
        self.admitted || (!self.billing_hold && !self.admin_approval_hold)
    }
}

/// Atomic association and admission queries.
pub trait InstallationLinkStore: Send + Sync {
    /// Atomically bind signed ownership and flow from the original requester's facts.
    fn associate_signed_installation(
        &self,
        signed: &SignedInstallation,
    ) -> Result<InstallationClaim>;
    /// Save entry evidence before exposing the GitHub install URL.
    fn begin_installation_attempt(&self, attempt: &InstallationAttempt) -> Result<()>;
    /// Resolve exact return state without inferring a workspace.
    fn installation_attempt(&self, digest: &[u8]) -> Result<Option<InstallationAttempt>>;
    /// Resolve exact sealed continuation rather than a person's newest attempt.
    fn installation_attempt_by_id(&self, id: Uuid) -> Result<Option<InstallationAttempt>>;
    /// Finish an own-user verified personal return without creating an organization claim.
    fn complete_personal_installation_attempt(
        &self,
        id: Uuid,
        requester: i64,
        installation: i64,
    ) -> Result<()>;
    /// Test whether a signed requester has unresolved Add evidence.
    fn has_pending_add(&self, requester_id: i64) -> Result<bool>;
    /// Preserve exact Add provenance even if later claim bookkeeping disappears.
    fn has_add_installation(&self, requester_id: i64, installation_id: i64) -> Result<bool>;
    /// Save proof and optional pending first/open result without replacing signed ownership or flow.
    fn save_installation_claim(
        &self,
        claim: &InstallationClaim,
        attempt: Option<Uuid>,
        result: InstallationResultDelivery,
    ) -> Result<InstallationClaim>;
    /// Read the exact requester's undelivered authorization results in stable claim order.
    fn pending_installation_results(
        &self,
        requester_id: i64,
    ) -> Result<Vec<PendingInstallationResult>>;
    /// Read one exact claim.
    fn installation_claim(&self, id: Uuid) -> Result<Option<InstallationClaim>>;
    /// Read an exact installation association.
    fn claim_for_installation(&self, installation_id: i64) -> Result<Option<InstallationClaim>>;
    /// Read one requester's pending and linked recovery outcomes.
    fn installation_claims(&self, requester_id: i64) -> Result<Vec<InstallationClaim>>;
    /// Save the exact current Admin offers from fresh memberships.
    fn offer_installation_workspaces(
        &self,
        claim_id: Uuid,
        requester_id: i64,
        workspaces: &[Uuid],
    ) -> Result<()>;
    /// Read previously authorized offers for this exact claim.
    fn installation_workspace_offers(&self, claim_id: Uuid, requester_id: i64)
    -> Result<Vec<Uuid>>;
    /// Choose one offered workspace, once. Replay cannot move the installation.
    fn choose_installation_workspace(
        &self,
        claim_id: Uuid,
        requester_id: i64,
        workspace_id: Uuid,
    ) -> Result<()>;
    /// Save link-time approval without changing a completed link's authority.
    fn authorize_installation_claim(&self, claim_id: Uuid, approved: bool) -> Result<()>;
    /// Read explicit local admission metadata for this exact registration.
    fn organization_activation(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
    ) -> Result<OrganizationActivation>;
    /// Test complete current execution authority for a registration.
    fn organization_runnable(&self, workspace_id: Uuid, organization_id: Uuid) -> Result<bool>;
    /// Clear billing holds when all linked organizations fit the current tier.
    fn reconcile_organization_billing(&self, workspace_id: Uuid) -> Result<()>;
    /// Record explicit current Admin approval, independent of original requester.
    fn approve_organization_activation(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
    ) -> Result<()>;
    /// Discover durable unfinished admission work after restart and billing changes.
    fn pending_organization_activations(&self) -> Result<Vec<Uuid>>;
    /// Discover signed Direct installs and verified Add choices awaiting registration.
    fn pending_installation_choices(&self) -> Result<Vec<Uuid>>;
}
