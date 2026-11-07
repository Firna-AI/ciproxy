//! Dashboard identity, access, and history-preserving management persistence.

mod continuations;
mod identity;
mod machines;
mod resources;
mod roles;
mod rows;
mod sessions;
mod takeouts;

use chrono::{DateTime, Utc};
use ciproxy_store_interface::dashboard::records::{
    AdminClaim, CurrentCostClock, CurrentMachineCost, LoginState, Member, Membership, Role,
    Session, Takeout, User,
};
use ciproxy_store_interface::dashboard::store::DashboardStore;
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::labels::Label;
use ciproxy_store_interface::metering::history::MachineCost;
use ciproxy_store_interface::scale_sets::ScaleSet;
use ciproxy_store_interface::workspaces::Workspace;
use uuid::Uuid;

use crate::connection::PgStore;

impl DashboardStore for PgStore {
    fn save_login_state(&self, state: &LoginState) -> Result<()> {
        sessions::save_state(self, state)
    }
    fn consume_login_state(&self, digest: &[u8], at: DateTime<Utc>) -> Result<Option<LoginState>> {
        sessions::consume_state(self, digest, at)
    }
    fn dashboard_user_by_github(&self, id: i64) -> Result<Option<User>> {
        identity::by_github(self, id)
    }
    fn dashboard_user(&self, id: Uuid) -> Result<Option<User>> {
        identity::user(self, id)
    }
    fn save_dashboard_user(&self, user: &User, memberships: &[Membership]) -> Result<()> {
        identity::save(self, user, memberships)
    }
    fn save_dashboard_credentials(&self, id: Uuid, credentials: &[u8]) -> Result<()> {
        identity::credentials(self, id, credentials)
    }
    fn establish_dashboard_session(
        &self,
        user: &User,
        memberships: &[Membership],
        session: &Session,
        installation_continuation: &[Uuid],
    ) -> Result<bool> {
        sessions::establish(self, user, memberships, session, installation_continuation)
    }
    fn dashboard_continuation(&self, session_id: Uuid, at: DateTime<Utc>) -> Result<Vec<Uuid>> {
        continuations::read(self, session_id, at)
    }
    fn dashboard_memberships(&self, id: Uuid) -> Result<Vec<Membership>> {
        identity::memberships(self, id)
    }
    fn dashboard_workspaces(&self, id: Uuid) -> Result<Vec<Workspace>> {
        identity::workspaces(self, id)
    }
    fn dashboard_users_due(&self, before: DateTime<Utc>) -> Result<Vec<User>> {
        identity::due(self, before)
    }
    fn expire_dashboard_auth(&self, at: DateTime<Utc>) -> Result<()> {
        sessions::expire(self, at)
    }
    fn dashboard_session(&self, digest: &[u8], at: DateTime<Utc>) -> Result<Option<Session>> {
        sessions::read(self, digest, at)
    }
    fn delete_dashboard_session(&self, id: Uuid) -> Result<()> {
        sessions::delete(self, id)
    }
    fn dashboard_members(&self, id: Uuid) -> Result<Vec<Member>> {
        roles::members(self, id)
    }
    fn dashboard_has_access_record(&self, github: i64) -> Result<bool> {
        roles::has_access_record(self, github)
    }
    fn dashboard_role(&self, id: Uuid, github: i64) -> Result<Option<Role>> {
        roles::role(self, id, github)
    }
    fn ensure_dashboard_initial_admin(
        &self,
        id: Uuid,
        github: i64,
        at: DateTime<Utc>,
    ) -> Result<()> {
        roles::first_admin(self, id, github, at)
    }
    fn save_dashboard_role(
        &self,
        id: Uuid,
        github: i64,
        role: Option<Role>,
        at: DateTime<Utc>,
    ) -> Result<()> {
        roles::save(self, id, github, role, at)
    }
    fn claim_dashboard_admin(
        &self,
        id: Uuid,
        user: Uuid,
        at: DateTime<Utc>,
        claim: AdminClaim,
    ) -> Result<bool> {
        roles::claim(self, id, user, at, claim)
    }
    fn rename_workspace(&self, id: Uuid, name: &str) -> Result<()> {
        resources::rename(self, id, name)
    }
    fn begin_organization_takeout(&self, id: Uuid, organization: Uuid) -> Result<Takeout> {
        takeouts::begin(self, id, organization)
    }
    fn pending_organization_takeouts(&self) -> Result<Vec<Takeout>> {
        takeouts::pending(self)
    }
    fn complete_organization_takeout(&self, takeout: &Takeout) -> Result<()> {
        takeouts::complete(self, takeout)
    }
    fn dashboard_machine_costs(&self, id: Uuid) -> Result<Vec<MachineCost>> {
        machines::costs(self, id)
    }
    fn set_dashboard_warm_runners(&self, id: Uuid, machine: Uuid, warm: i32) -> Result<()> {
        machines::warm(self, id, machine, warm)
    }
    fn save_dashboard_machine_cost(
        &self,
        id: Uuid,
        cost: &CurrentMachineCost,
        clock: &dyn CurrentCostClock,
    ) -> Result<()> {
        machines::cost(self, id, cost, clock)
    }
    fn dashboard_scale_sets(&self, id: Uuid, organization: Uuid) -> Result<Vec<ScaleSet>> {
        resources::sets(self, id, organization)
    }
    fn retire_dashboard_label(&self, id: Uuid, label: Uuid, at: DateTime<Utc>) -> Result<()> {
        resources::retire(self, id, label, at)
    }
    fn retired_label_workspaces(&self) -> Result<Vec<Uuid>> {
        resources::retired_workspaces(self)
    }
    fn retired_dashboard_labels(&self, id: Uuid) -> Result<Vec<Label>> {
        resources::retired(self, id)
    }
}
