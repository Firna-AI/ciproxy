//! Credential-safe dashboard persistence values.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// A person verified through GitHub user authorization.
#[derive(Clone)]
pub struct User {
    /// Stable internal person identity.
    pub id: Uuid,
    /// Stable GitHub account identity.
    pub github_id: i64,
    /// Current GitHub account name.
    pub login: String,
    /// GitHub profile image URL.
    pub avatar_url: String,
    /// Encrypted OAuth grant, bound to this internal identity.
    pub credentials: Vec<u8>,
    /// Last successful complete membership observation.
    pub checked_at: DateTime<Utc>,
}

/// A current organization membership verified with the person's user token.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Membership {
    /// Stable GitHub organization identity.
    pub github_id: i64,
    /// Whether the person owns the organization.
    pub owner: bool,
}

/// A digest-authenticated fixed-duration dashboard session.
#[derive(Clone)]
pub struct Session {
    /// Internal session identifier.
    pub id: Uuid,
    /// The authenticated person.
    pub user_id: Uuid,
    /// SHA-256 digest of the opaque browser cookie.
    pub digest: Vec<u8>,
    /// SHA-256 digest of the associated CSRF token.
    pub csrf_digest: Vec<u8>,
    /// Exclusive end of the 30-day sign-in.
    pub expires_at: DateTime<Utc>,
}

/// A short-lived one-use OAuth attempt bound to a browser cookie.
#[derive(Clone)]
pub struct LoginState {
    /// Identity used to bind the encrypted PKCE verifier.
    pub id: Uuid,
    /// SHA-256 digest of the state and browser cookie.
    pub digest: Vec<u8>,
    /// Encrypted PKCE verifier.
    pub verifier: Vec<u8>,
    /// Exclusive validity boundary.
    pub expires_at: DateTime<Utc>,
}

/// A workspace role or a pending request for access.
#[derive(Clone, Debug)]
pub struct Member {
    /// Stable GitHub account identity.
    pub github_user_id: i64,
    /// Current login, absent until this person first signs in.
    pub login: Option<String>,
    /// Admin, Billing, Viewer, or an unanswered request.
    pub role: Option<Role>,
    /// When the person asked for access or received the first role.
    pub requested_at: DateTime<Utc>,
}

/// The nested workspace roles.
#[derive(Clone, Copy, Debug, Eq, PartialEq, strum::EnumString, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum Role {
    /// Full workspace control, including runner secrets.
    Admin,
    /// Read access and billing management.
    Billing,
    /// Read access to jobs, machines, savings, and usage.
    Viewer,
}

/// A pending remote cleanup after intentional organization removal.
#[derive(Clone, Debug)]
pub struct Takeout {
    /// Historical registration kept in its original workspace.
    pub organization_id: Uuid,
    /// Workspace that owns the historical registration.
    pub workspace_id: Uuid,
    /// Exact installation to uninstall, never a later reinstall.
    pub installation_id: i64,
}

/// Which already-proved owner authority an atomic Admin claim may use.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdminClaim {
    /// Require ownership of every installed organization.
    AllOrganizations,
    /// Also allow an owner when no currently eligible Admin remains.
    Recovery,
}
