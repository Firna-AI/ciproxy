//! GitHub sign-in, verified memberships, and reusable workspace authorization.

mod adapters;
pub(crate) mod authentication;
mod flow;
mod install_authority;
mod link_choice;
mod link_continuation;
mod link_identity;
mod links;
mod pending;
mod setup;
mod sign_in;
pub mod workspace;

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ciproxy_github::app::GitHubApp;
use ciproxy_github::installation_state::InstalledApp;
use ciproxy_store_interface::dashboard::records::{Session, User};
use ciproxy_store_interface::installation_links::InstallationClaim;
use ciproxy_store_interface::store::Store;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;
use uuid::Uuid;

use self::workspace::WorkspaceId;
use crate::crypto::CredentialCipher;
use crate::error::Result;
use crate::identity::UserIdentity;
use crate::runtime::Runtime;
use crate::wire::access::{RemovedOrganization, SetupOutcome};

/// A protected OAuth attempt to be returned only to its browser.
pub struct LoginAttempt {
    /// GitHub authorization URL.
    pub url: String,
    /// Opaque browser-binding cookie value.
    pub binding: SecretString,
}

/// A successful fixed-duration sign-in and its CSRF token.
pub struct LoginSuccess {
    /// Opaque or encrypted HttpOnly session cookie.
    pub cookie: SecretString,
    /// Readable token required in the CSRF header of mutations.
    pub csrf: SecretString,
    /// Fixed exclusive sign-in expiry.
    pub expires_at: DateTime<Utc>,
}

/// Exact install continuation kept only in sealed browser metadata.
#[derive(Clone, Deserialize, Serialize)]
pub(crate) enum InstallContext {
    /// Exact explicit Direct or Add entry belonging to this person.
    Attempt(Uuid),
    /// Missing flow evidence cannot become a Direct retry.
    AmbiguousAdd,
}

/// An authenticated browser, with credentials excluded from Debug.
pub struct Principal {
    pub(crate) installation_continuation: Vec<Uuid>,
    pub(crate) install_context: Option<InstallContext>,
    pub(crate) user: User,
    pub(crate) session: Session,
    pub(crate) pending: bool,
    pub(crate) had_access: bool,
    pub(crate) removed_organization: Option<RemovedOrganization>,
    pub(crate) csrf: String,
    pub(crate) setup: Option<SetupOutcome>,
}

impl Principal {
    /// Whether this sign-in has no retained private workspace session.
    pub fn browser_only(&self) -> bool {
        self.pending
    }
    /// Whether this sign-in previously held a durable workspace role or request.
    pub fn had_access(&self) -> bool {
        self.had_access
    }
    /// Most recent removed organization verified during fresh GitHub authorization.
    pub fn removed_organization(&self) -> Option<&RemovedOrganization> {
        self.removed_organization.as_ref()
    }
    /// Verified setup metadata without a frontend installation-ID bridge.
    pub fn setup(&self) -> Option<&SetupOutcome> {
        self.setup.as_ref()
    }
    /// Exact claims proved or completed during this browser's latest authorization.
    pub fn installation_continuation(&self) -> &[Uuid] {
        &self.installation_continuation
    }
    /// CSRF value restored from the sealed browser sign-in.
    pub fn csrf(&self) -> &str {
        &self.csrf
    }
    /// Stable internal person identity used by the machine gateway.
    pub fn id(&self) -> Uuid {
        self.user.id
    }
    /// Stable GitHub person identity used by role and install storage.
    pub fn github_id(&self) -> i64 {
        self.user.github_id
    }
    /// Current GitHub login for the account menu.
    pub fn login(&self) -> &str {
        &self.user.login
    }
    /// Fixed exclusive dashboard sign-in expiry.
    pub fn expires_at(&self) -> DateTime<Utc> {
        self.session.expires_at
    }
    /// Authenticate a mutation's CSRF header against this session.
    pub fn accepts_csrf(&self, value: &str) -> bool {
        bool::from(authentication::digest(value).ct_eq(&self.session.csrf_digest))
    }
}

/// Dashboard identity operations consumed by the HTTP router through dyn.
#[async_trait]
pub trait DashboardAccess: Send + Sync {
    /// Start a one-use browser-bound GitHub App authorization attempt.
    async fn begin_login(&self, previous: Option<&str>) -> Result<LoginAttempt>;
    /// Start setup proof for an installation read through the configured App.
    async fn begin_setup(
        &self,
        installation: Option<InstalledApp>,
        install_state: Option<String>,
        request_only: bool,
        previous: Option<&str>,
    ) -> Result<LoginAttempt>;
    /// Finish only the matching browser's unconsumed OAuth attempt.
    async fn finish_login(
        &self,
        code: SecretString,
        state: &str,
        cookie: &str,
        github: &dyn GitHubApp,
    ) -> Result<LoginSuccess>;
    /// Recheck owned installed claims with fresh user proof and return their states.
    async fn refresh_installation_links(
        &self,
        principal: &Principal,
        github: &dyn GitHubApp,
    ) -> Result<Vec<InstallationClaim>>;
    /// Verify only the selected claim, preserving exact prior offer authority.
    async fn choose_installation_link(
        &self,
        principal: &Principal,
        claim_id: Uuid,
        workspace_id: Uuid,
        github: &dyn GitHubApp,
    ) -> Result<()>;
    /// Authenticate and refresh memberships when their hourly check is due.
    async fn authenticate(&self, cookie: &str) -> Result<Principal>;
    /// End the current browser's durable sign-in.
    async fn sign_out(&self, principal: &Principal) -> Result<()>;
    /// Refresh all due identities and prune expired authentication state.
    async fn reconcile(&self) -> Result<()>;
    /// Refresh stale Admin memberships before an owner-recovery decision.
    async fn refresh_workspace_admins(&self, workspace_id: Uuid) -> Result<()>;
    /// Require read access or the Admin role in exactly this workspace.
    async fn require_role(
        &self,
        principal: &Principal,
        workspace_id: WorkspaceId,
        admin: bool,
    ) -> Result<Uuid>;
    /// Open a workspace for the existing owner-claim authorization path.
    async fn open_claim_workspace(
        &self,
        principal: &Principal,
        workspace_id: WorkspaceId,
    ) -> Result<Uuid>;
}

/// Shared identity service constructed before the gateway and GitHub App.
pub struct AccessService {
    store: Arc<dyn Store>,
    identity: Arc<dyn UserIdentity>,
    cipher: Arc<dyn CredentialCipher>,
    runtime: Arc<dyn Runtime>,
}

impl AccessService {
    /// Construct once and coerce cloned Arcs to all three access adapters.
    pub fn new(
        store: Arc<dyn Store>,
        identity: Arc<dyn UserIdentity>,
        cipher: Arc<dyn CredentialCipher>,
        runtime: Arc<dyn Runtime>,
    ) -> Self {
        Self {
            store,
            identity,
            cipher,
            runtime,
        }
    }
}
