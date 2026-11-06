//! Atomic GitHub lifecycle transitions and immutable ordered notification snapshots.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::Result;
use crate::github::Repository;
use crate::labels::Label;
use crate::organizations::{Organization, RepositoryAccess};
use crate::scale_sets::ScaleSet;
use crate::workspaces::Workspace;

/// Remote resources prepared before activation can be committed or delivered.
#[derive(Clone, Debug)]
pub struct GithubPreparedResources {
    /// The visible repositories and default branches.
    pub repositories: Vec<Repository>,
    /// Current remote identities for all labels of the organization.
    pub scale_sets: Vec<ScaleSet>,
}

/// The requested current App state and any required completed preparation.
#[derive(Clone, Debug)]
pub enum GithubLifecycleTarget {
    /// Reconciliation completed for this activation.
    Active(GithubPreparedResources),
    /// Suspension needs no runner preparation.
    Suspended,
    /// The App is installed but local admission blocks all runner preparation.
    Inactive,
}

/// A new or retained organization registration and its prepared lifecycle state.
#[derive(Clone, Debug)]
pub struct GithubRegistration {
    /// The new workspace, absent when joining an existing workspace.
    pub workspace: Option<Workspace>,
    /// The requested current or retained organization registration.
    pub organization: Organization,
    /// The default labels for a new workspace only.
    pub defaults: Vec<Label>,
    /// The signed requester's pending first-Admin assignment.
    pub initial_admin: i64,
    /// Preparation for the resulting lifecycle state.
    pub target: GithubLifecycleTarget,
}

/// A verified installation change and its optional repository access edit.
#[derive(Clone, Debug)]
pub struct GithubLifecycleUpdate {
    /// The stable GitHub organization identity.
    pub github_id: i64,
    /// The current installation identity verified by GitHub.
    pub installation_id: i64,
    /// The current organization login used by API routes.
    pub login: String,
    /// A repository access change, absent when preserving the saved choice.
    pub repositories: Option<RepositoryAccess>,
    /// Preparation for the resulting lifecycle state.
    pub target: GithubLifecycleTarget,
    /// Whether an explicit lifecycle event also requests current-state delivery.
    pub notify: bool,
}

/// An immutable lifecycle notification saved with its committed transition.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum LifecycleEvent {
    /// Resume listeners and allow prepared runner starts.
    Active(Organization),
    /// Stop idle runners while preserving running jobs and workspace resources.
    Suspended(Organization),
    /// Local admission prevents new execution without changing GitHub App state.
    Inactive {
        /// Exact linked registration.
        organization: Organization,
        /// Current tier cannot accommodate the linked organization count.
        billing_hold: bool,
        /// A current Admin has not approved the link.
        admin_approval_hold: bool,
    },
    /// Stop idle capacity and perform removal cleanup for the saved workspace.
    Removed {
        /// The removed installation's exact committed organization snapshot.
        organization: Organization,
        /// Whether this removal left its workspace without an installed App.
        last_organization: bool,
    },
}

impl LifecycleEvent {
    /// The organization snapshot, independent of later row changes or deletion.
    pub fn organization(&self) -> &Organization {
        match self {
            Self::Active(organization)
            | Self::Suspended(organization)
            | Self::Removed { organization, .. }
            | Self::Inactive { organization, .. } => organization,
        }
    }
}

/// One immutable notification in its organization's delivery order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleDelivery {
    /// The monotonic delivery identity, distinct from organization row identity.
    pub sequence: i64,
    /// The exact committed event, never reconstructed from a mutable row.
    pub event: LifecycleEvent,
}

/// Persistence for atomic transitions and durable ordered lifecycle delivery.
///
/// Hold the existing organization mutation lease for transitions and delivery.
/// Only pending-organization discovery may run without that lease.
pub trait GithubLifecycleStore: Send + Sync {
    /// Commit registration, prepared bindings, and its lifecycle event together.
    fn register_github_installation(
        &self,
        registration: &GithubRegistration,
    ) -> Result<Organization>;
    /// Commit verified state, optional access/bindings, and its event together.
    fn update_github_installation(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
        update: &GithubLifecycleUpdate,
    ) -> Result<Organization>;
    /// Commit removal, credential bookkeeping, and immutable cleanup together.
    ///
    /// Already removed registrations keep their original event and deadline.
    fn remove_github_installation(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
        installation_id: i64,
        at: DateTime<Utc>,
    ) -> Result<Organization>;
    /// Find organizations with pending events, including purged registrations.
    fn pending_github_lifecycle_organizations(&self) -> Result<Vec<i64>>;
    /// Read pending immutable events in increasing sequence order.
    fn pending_github_lifecycle(&self, github_id: i64) -> Result<Vec<LifecycleDelivery>>;
    /// Delete an accepted event under the organization's mutation lease.
    ///
    /// Confirm only after consumer acceptance. A crash before confirmation
    /// causes delivery again, so the consumer must be idempotent.
    fn confirm_github_lifecycle(&self, github_id: i64, sequence: i64) -> Result<()>;
}
