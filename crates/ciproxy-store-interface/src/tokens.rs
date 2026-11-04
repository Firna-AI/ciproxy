//! Hash-only install, machine, and sandbox credential storage.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::Result;
use crate::secrets::TokenHash;

/// The one authority granted by a token.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenSubject {
    /// Register machines in the workspace.
    Install,
    /// Connect as one registered machine.
    Machine(Uuid),
    /// Report the end of one sandbox runner.
    Sandbox {
        /// The organization that owns this sandbox runner.
        organization_id: Uuid,
        /// The one runner whose sandbox may report its end.
        runner_id: Uuid,
    },
}

/// Credential metadata returned after hash authentication.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token {
    /// The token record identifier.
    pub id: Uuid,
    /// The workspace authorized by the token.
    pub workspace_id: Uuid,
    /// The restricted token authority.
    pub subject: TokenSubject,
    /// When the token was issued.
    pub created_at: DateTime<Utc>,
    /// The exclusive expiry boundary, absent for machine credentials.
    pub expires_at: Option<DateTime<Utc>>,
    /// When the token was revoked.
    pub revoked_at: Option<DateTime<Utc>>,
}

/// Create, authenticate, and revoke credentials without plaintext tokens.
pub trait TokenStore: Send + Sync {
    /// Store a reusable install token with an exact one-hour lifetime.
    fn create_install_token(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        hash: &TokenHash,
        at: DateTime<Utc>,
    ) -> Result<()>;
    /// Store a credential limited to one machine or sandbox runner.
    fn create_scoped_token(&self, token: &Token, hash: &TokenHash) -> Result<()>;
    /// Find an unrevoked, unexpired token by its precomputed hash.
    ///
    /// This authentication lookup returns the token's workspace. All
    /// subsequent resource queries must use that workspace identifier.
    fn authenticate_token(&self, hash: &TokenHash, at: DateTime<Utc>) -> Result<Option<Token>>;
    /// Revoke one token inside its workspace.
    fn revoke_token(&self, workspace_id: Uuid, id: Uuid, at: DateTime<Utc>) -> Result<()>;
}
