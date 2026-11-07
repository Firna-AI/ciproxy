//! Sign-in, workspace, and member payloads.

use crate::access::workspace::WorkspaceId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

/// Stable workspace role on the HTTP boundary.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceRole {
    /// Full workspace control.
    Admin,
    /// Viewer permissions plus billing control.
    Billing,
    /// Read-only workspace access.
    Viewer,
}

/// The fixed-duration dashboard sign-in and accessible workspaces.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct SessionView {
    /// Exact claim IDs from the latest successful GitHub authorization.
    /// Includes proved or completed results recovered from a failed authorization.
    /// All IDs refer to organization_links. No order selects a continuation.
    /// Reloads keep this list. A successful authorization replaces it, even when empty.
    pub installation_continuation: Vec<Uuid>,
    /// Current requester-owned installation recovery and workspace choices.
    pub organization_links: Vec<super::links::OrganizationLinkView>,
    /// Signed-in access state for install, access, and left pages.
    pub state: SignInState,
    /// CSRF token for requests that change data.
    pub csrf_token: String,
    /// Same-origin top-level navigation for fresh GitHub authorization.
    pub reauthorization_url: Option<String>,
    /// Whether a newly available role/request needs fresh GitHub membership proof.
    pub reauthorization_required: bool,
    /// Current GitHub login.
    pub login: String,
    /// Exclusive fixed sign-in expiry.
    pub expires_at: DateTime<Utc>,
    /// Workspaces whose organizations contain this person.
    pub workspaces: Vec<WorkspaceView>,
    /// Verified setup return, recorded only after fresh user association proof.
    pub setup: Option<SetupOutcome>,
    /// Requests owned by this exact GitHub person, awaiting an organization owner.
    pub pending_install_requests: Vec<PendingInstallation>,
    /// Most recent owner removal among the memberships verified at authorization.
    pub removed_organization: Option<RemovedOrganization>,
    /// Install entry point when no workspace is available.
    pub install_url: String,
}

/// Workspace access, recovery, and soft-limit state.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct WorkspaceView {
    /// Stable workspace identity.
    pub id: String,
    /// Customer-visible name.
    pub name: String,
    /// This person's role, absent while access is pending.
    pub role: Option<WorkspaceRole>,
    /// Admin names shown to members who request access.
    pub admins: Vec<String>,
    /// Whether an owner claim can recover the Admin role.
    pub can_claim_admin: bool,
    /// Whether this workspace exceeds Free's organization soft limit.
    pub organization_limit_exceeded: bool,
}

/// Current roles and pending requests for the Settings Members tab.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct MembersView {
    /// People with roles or unanswered access requests.
    pub members: Vec<MemberView>,
    /// Count displayed on the Members tab.
    pub open_requests: u32,
    /// Trust notice used before an Admin grant.
    pub admin_notice: String,
}

/// One workspace member or access request.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct MemberView {
    /// Stable GitHub account identifier as a string.
    pub id: String,
    /// Account name, absent before its first dashboard sign-in.
    pub login: Option<String>,
    /// Granted role, absent for an open request.
    pub role: Option<WorkspaceRole>,
    /// Initial request or grant time in UTC.
    pub requested_at: DateTime<Utc>,
}

/// A role grant or change. The Admin trust notice precedes this request.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct RoleChange {
    /// The granted nested workspace role.
    pub role: WorkspaceRole,
}

/// OAuth callback parameters returned by GitHub.
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct OAuthCallback {
    /// One-use authorization code.
    pub code: Option<String>,
    /// Browser-bound state.
    pub state: Option<String>,
    /// GitHub cancellation or refusal.
    pub error: Option<String>,
}

/// One tenant-scoped path parameter.
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct WorkspacePath {
    /// Stable workspace identity.
    #[param(value_type=Uuid)]
    pub workspace_id: WorkspaceId,
}

/// A workspace and one of its members.
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct MemberPath {
    /// Stable workspace identity.
    #[param(value_type=Uuid)]
    pub workspace_id: WorkspaceId,
    /// Stable GitHub account identity as a string.
    pub member_id: String,
}

/// Empty success payload for idempotent mutations.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct Success {
    /// The mutation completed successfully.
    pub ok: bool,
}

/// Sign-in remains valid when no private workspace session can be retained.
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SignInState {
    /// A retained session can read its permitted workspaces.
    Ready,
    /// No workspace access record exists yet.
    InstallRequired,
    /// A verified organization removed the App and retains its reinstall window.
    AppRemoved,
    /// A previous role or request ended, without ending GitHub sign-in.
    AccessEnded,
}

/// Untrusted parameters returned to the registered App setup URL.
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct SetupCallback {
    /// Candidate installation identity. It grants no authority.
    pub installation_id: Option<String>,
    /// Untrusted install-flow state. It never selects identity or workspace.
    pub state: Option<String>,
    /// GitHub can return a request that has not become an installation yet.
    pub setup_action: Option<String>,
}

/// Token-free setup outcome proved by configured-App and fresh user APIs.
#[derive(Clone, Deserialize, Serialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SetupOutcome {
    /// The original person's request still waits for owner approval.
    WaitingApproval,
    /// An organization installation visible to its current organization member.
    Organization {
        /// Exact configured App installation identity.
        installation_id: String,
        /// GitHub organization identity.
        github_id: String,
        /// Current GitHub organization login.
        login: String,
        /// Whether GitHub has suspended the installation.
        suspended: bool,
    },
    /// Personal installation requires an organization, without a workspace.
    OrganizationsOnly {
        /// Exact personal installation identity.
        installation_id: String,
        /// GitHub settings link that removes this installation.
        remove_url: String,
    },
}

/// An organization installation request that an owner has not approved.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct PendingInstallation {
    /// Stable GitHub request identity as a decimal string.
    pub id: String,
    /// Organization identity as a decimal string.
    pub github_id: String,
    /// Organization name chosen on GitHub.
    pub organization: String,
    /// Time GitHub received the request.
    pub created_at: DateTime<Utc>,
}

/// Browser headers required by every authenticated mutation.
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Header)]
pub struct MutationHeaders {
    /// Exact configured dashboard origin.
    #[serde(rename = "Origin")]
    pub origin: String,
    /// The session's CSRF token returned by the session endpoint.
    #[serde(rename = "x-ciproxy-csrf")]
    pub csrf: String,
}

/// Organization removal metadata verified through this person's GitHub membership.
#[derive(Clone, Deserialize, Serialize, ToSchema)]
pub struct RemovedOrganization {
    /// Stable GitHub organization identity as a decimal string.
    pub github_id: String,
    /// Organization name shown on the removed-App page.
    pub login: String,
    /// Exclusive deadline for a reinstall that preserves retained history.
    pub delete_after: DateTime<Utc>,
}
