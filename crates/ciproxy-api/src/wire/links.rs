//! Requester-owned installation choice and independent activation reasons.

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

/// One exact recovery route.
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in=Path)]
pub struct ClaimPath {
    /// Stable requester-owned claim.
    pub claim_id: Uuid,
}

/// Explicit selection of a previously offered workspace.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct WorkspaceChoice {
    /// Exact offered workspace.
    pub workspace_id: Uuid,
}

/// Top-level navigation to GitHub's unchanged organization chooser.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct InstallLink {
    /// Random attempt identity, not workspace selection authority.
    pub attempt_id: Uuid,
    /// Configured installation URL carrying only opaque state.
    pub url: String,
}

/// Observable progress of one installed organization association.
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum LinkState {
    /// The browser proved installation but its signed event is not yet associated.
    ConfirmingInstallation,
    /// Original requester must authorize with GitHub again.
    ReauthorizationRequired,
    /// Several current Admin workspaces need an explicit selection.
    ChooseWorkspace,
    /// No current eligible workspace is available.
    WaitingForAdmin,
    /// The exact saved choice waits for registration.
    Linking,
    /// The organization belongs to its workspace but has local admission holds.
    LinkedInactive,
    /// The organization is linked and admitted.
    Linked,
}

/// An independent local admission blocker.
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ActivationReason {
    /// The linked organization count needs another tier.
    BillingUpgrade,
    /// A current Admin must approve the saved link.
    AdminApproval,
}

/// Exact workspace and minimum tier for a supported billing action.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct OrganizationUpgrade {
    /// Workspace whose confirmed plan must change.
    pub workspace_id: Uuid,
    /// Minimum tier that supports several organizations.
    pub tier: OrganizationTier,
}

/// Tier needed to admit several linked organizations.
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum OrganizationTier {
    /// Several organizations are available on Team.
    Team,
}

/// Local execution state, independent of the GitHub App state.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct ActivationView {
    /// Initial admission completed and will survive a later downgrade.
    pub admitted: bool,
    /// Both reasons can exist at the same time.
    pub reasons: Vec<ActivationReason>,
    /// Server is retrying remote preparation after authorization completed.
    pub preparing: bool,
    /// Original requester's GitHub identity, absent for legacy direct installs.
    pub added_by: Option<String>,
    /// Current Admin can approve when no billing blocker remains.
    pub can_enable: bool,
    /// Product action for a billing hold.
    pub upgrade: Option<OrganizationUpgrade>,
    /// Ask an existing Admin when this person cannot manage the required action.
    pub ask_admin: bool,
}

/// An offered destination backed by fresh own-user membership proof.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct EligibleWorkspace {
    /// Exact destination.
    pub id: Uuid,
    /// Workspace name.
    pub name: String,
    /// Selecting it currently creates a billing hold.
    pub requires_upgrade: bool,
}

/// Complete typed picker/recovery result for the original requester.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct OrganizationLinkView {
    /// Reading metadata does not authorize use of a stored GitHub grant.
    pub reauthorization_required: bool,
    /// Stable claim identity.
    pub id: Uuid,
    /// Verified installation identity.
    pub installation_id: String,
    /// Organization name.
    pub organization: String,
    /// Current progress.
    pub state: LinkState,
    /// Current eligible choices, with no default selection.
    pub workspaces: Vec<EligibleWorkspace>,
    /// Saved exact destination.
    pub workspace_id: Option<Uuid>,
    /// Linked registration.
    pub organization_id: Option<Uuid>,
    /// Linked local state, absent before registration.
    pub activation: Option<ActivationView>,
    /// Fresh authorization action for this original person.
    pub reauthorization_url: String,
}
