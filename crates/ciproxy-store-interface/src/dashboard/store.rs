//! Query boundary for dashboard identity and access state.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::dashboard::records::{
    AdminClaim, CurrentCostClock, CurrentMachineCost, LoginState, Member, Membership, Role,
    Session, Takeout, User,
};
use crate::error::Result;
use crate::labels::Label;
use crate::metering::history::MachineCost;
use crate::scale_sets::ScaleSet;
use crate::workspaces::Workspace;

/// Durable queries consumed by the dashboard service through dyn Store.
pub trait DashboardStore: Send + Sync {
    /// Persist a one-use browser-bound OAuth state.
    fn save_login_state(&self, state: &LoginState) -> Result<()>;
    /// Consume one valid OAuth state atomically.
    fn consume_login_state(&self, digest: &[u8], at: DateTime<Utc>) -> Result<Option<LoginState>>;
    /// Find a previously authorized person by stable GitHub identity.
    fn dashboard_user_by_github(&self, github_id: i64) -> Result<Option<User>>;
    /// Read one person's encrypted grant and last membership check.
    fn dashboard_user(&self, id: Uuid) -> Result<Option<User>>;
    /// Save a grant and complete membership snapshot, removing unsupported roles.
    fn save_dashboard_user(&self, user: &User, memberships: &[Membership]) -> Result<()>;
    /// Save a rotated encrypted pair without changing membership freshness or roles.
    fn save_dashboard_credentials(&self, user_id: Uuid, credentials: &[u8]) -> Result<()>;
    /// Commit identity, memberships, access requests, and its session together.
    fn establish_dashboard_session(
        &self,
        user: &User,
        memberships: &[Membership],
        session: &Session,
    ) -> Result<bool>;
    /// Read the verified current membership snapshot.
    fn dashboard_memberships(&self, user_id: Uuid) -> Result<Vec<Membership>>;
    /// Read workspaces whose organizations still contain this person.
    fn dashboard_workspaces(&self, user_id: Uuid) -> Result<Vec<Workspace>>;
    /// Read identities whose complete membership check is due.
    fn dashboard_users_due(&self, before: DateTime<Utc>) -> Result<Vec<User>>;
    /// Delete expired OAuth attempts and sessions, and unneeded person records.
    fn expire_dashboard_auth(&self, at: DateTime<Utc>) -> Result<()>;
    /// Authenticate a cookie digest before its exclusive expiration.
    fn dashboard_session(&self, digest: &[u8], at: DateTime<Utc>) -> Result<Option<Session>>;
    /// End one session without changing other browser sign-ins.
    fn delete_dashboard_session(&self, id: Uuid) -> Result<()>;
    /// Read roles and access requests for one workspace.
    fn dashboard_members(&self, workspace_id: Uuid) -> Result<Vec<Member>>;
    /// Test whether a browser-only identity has a role/request ready for fresh proof.
    fn dashboard_has_access_record(&self, github_user_id: i64) -> Result<bool>;
    /// Read a role only while current verified organization membership supports it.
    fn dashboard_role(&self, workspace_id: Uuid, github_user_id: i64) -> Result<Option<Role>>;
    /// Idempotently save the first installer's Admin grant before their first sign-in.
    fn ensure_dashboard_initial_admin(
        &self,
        workspace_id: Uuid,
        github_user_id: i64,
        at: DateTime<Utc>,
    ) -> Result<()>;
    /// Change a current member's role, or decline/remove their access.
    fn save_dashboard_role(
        &self,
        workspace_id: Uuid,
        github_user_id: i64,
        role: Option<Role>,
        at: DateTime<Utc>,
    ) -> Result<()>;
    /// Atomically claim Admin when this owner meets the documented recovery rule.
    fn claim_dashboard_admin(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        at: DateTime<Utc>,
        claim: AdminClaim,
    ) -> Result<bool>;
    /// Rename a workspace without changing its organization ownership.
    fn rename_workspace(&self, workspace_id: Uuid, name: &str) -> Result<()>;
    /// Stop new starts and save intentional removal plus cleanup delivery atomically.
    /// This refuses the last organization and preserves history without retention.
    fn begin_organization_takeout(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
    ) -> Result<Takeout>;
    /// Read unfinished intentional removals after a restart.
    fn pending_organization_takeouts(&self) -> Result<Vec<Takeout>>;
    /// Confirm remote cleanup of the exact removed installation.
    fn complete_organization_takeout(&self, takeout: &Takeout) -> Result<()>;
    /// Read machine cost intervals for machine details.
    fn dashboard_machine_costs(&self, workspace_id: Uuid) -> Result<Vec<MachineCost>>;
    /// Change only the warm target of an active machine, without cost history.
    fn set_dashboard_warm_runners(
        &self,
        workspace_id: Uuid,
        machine_id: Uuid,
        warm_runners: i32,
    ) -> Result<()>;
    /// Read the clock under the locks and use that time for both interval boundaries.
    /// This never changes the warm target or supplies earlier unknown prices.
    fn save_dashboard_machine_cost(
        &self,
        workspace_id: Uuid,
        cost: &CurrentMachineCost,
        clock: &dyn CurrentCostClock,
    ) -> Result<()>;
    /// Read active and historical bindings for remote deletion only.
    fn dashboard_scale_sets(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
    ) -> Result<Vec<ScaleSet>>;
    /// Hide a deleted label while preserving job and runner references.
    fn retire_dashboard_label(
        &self,
        workspace_id: Uuid,
        label_id: Uuid,
        at: DateTime<Utc>,
    ) -> Result<()>;
    /// Discover workspaces with retired labels whose remote cleanup is idempotent.
    fn retired_label_workspaces(&self) -> Result<Vec<Uuid>>;
    /// Read retired labels for idempotent remote scale-set cleanup.
    fn retired_dashboard_labels(&self, workspace_id: Uuid) -> Result<Vec<Label>>;
}
