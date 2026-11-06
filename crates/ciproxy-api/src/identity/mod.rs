//! GitHub App user authorization and the person's own organization memberships.

pub mod http;
mod tokens;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ciproxy_store_interface::dashboard::records::Membership;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

use crate::error::{ErrorResultExt, Result};

/// Credential-free authenticated GitHub profile.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Profile {
    /// Stable GitHub account identity.
    pub id: i64,
    /// Current GitHub account name.
    pub login: String,
    /// Profile image URL from GitHub.
    pub avatar_url: String,
}

/// A user OAuth grant. Debug hides all token bytes.
#[derive(Debug, Deserialize)]
pub struct Grant {
    /// Current user access token.
    pub access_token: SecretString,
    /// Exclusive access-token expiration, absent for a non-expiring grant.
    pub expires_at: Option<DateTime<Utc>>,
    /// Refresh token returned by GitHub for expiring user grants.
    pub refresh_token: Option<SecretString>,
    /// Exclusive refresh-token expiration.
    pub refresh_expires_at: Option<DateTime<Utc>>,
}

impl Grant {
    /// Serialize for encryption without exposing credential bytes in Debug.
    pub fn encode(&self) -> Result<SecretString> {
        tokens::encode(self)
    }
    /// Read a decrypted grant at the credential boundary.
    pub fn decode(value: &SecretString) -> Result<Self> {
        serde_json::from_str(value.expose_secret()).internal()
    }
}

/// GitHub authorization boundary, never a GitHub App Members permission.
#[async_trait]
pub trait UserIdentity: Send + Sync {
    /// Build the App authorization URL with browser-bound state and PKCE S256.
    fn authorization_url(&self, state: &str, challenge: &str) -> String;
    /// Exchange one authorization code and the matching PKCE verifier.
    async fn exchange(&self, code: SecretString, verifier: SecretString) -> Result<Grant>;
    /// Refresh an expiring user grant without extending dashboard sign-in.
    async fn refresh(&self, grant: Grant) -> Result<Grant>;
    /// Read the authenticated person's current stable identity.
    async fn profile(&self, grant: &Grant) -> Result<Profile>;
    /// Read all pages of active memberships using that person's user grant.
    async fn memberships(&self, grant: &Grant) -> Result<Vec<Membership>>;
}
