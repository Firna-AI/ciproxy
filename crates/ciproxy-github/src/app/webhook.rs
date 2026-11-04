//! Typed installation webhook payloads and constant-time HMAC verification.

use async_trait::async_trait;
use hmac::{Hmac, KeyInit, Mac};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use sha2::Sha256;

use crate::error::{Error, Result, ResultExt};

/// GitHub actions relevant to an App installation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InstallationAction {
    /// The App was installed or an installation request was approved.
    Created,
    /// GitHub removed the App and revoked its access.
    Deleted,
    /// GitHub suspended the installation.
    Suspend,
    /// GitHub restored the suspended installation.
    Unsuspend,
    /// GitHub accepted changed App permissions.
    NewPermissionsAccepted,
    /// A lifecycle action which this service does not use.
    #[serde(other)]
    Other,
}

/// The GitHub account kind, independent of its login text.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
pub enum AccountKind {
    /// A GitHub organization.
    Organization,
    /// A personal GitHub account.
    User,
    /// An unsupported account type.
    #[serde(other)]
    Other,
}

/// The account that owns the installation.
#[derive(Clone, Debug, Deserialize)]
pub struct Account {
    /// The GitHub account identity.
    pub id: i64,
    /// The current GitHub account login.
    pub login: String,
    /// GitHub's account type.
    #[serde(rename = "type")]
    pub kind: AccountKind,
}

/// Installation identity from GitHub's authenticated webhook.
#[derive(Clone, Debug, Deserialize)]
pub struct Installation {
    /// The GitHub installation identity.
    pub id: i64,
    /// The account on which GitHub installed the App.
    pub account: Account,
}

/// A GitHub person identified by an authenticated webhook.
#[derive(Clone, Debug, Deserialize)]
pub struct Actor {
    /// The stable GitHub user identity.
    pub id: i64,
}

/// The fields used from the installation lifecycle webhook.
#[derive(Clone, Debug, Deserialize)]
pub struct InstallationEvent {
    /// The lifecycle change.
    pub action: InstallationAction,
    /// The installation and its target account.
    pub installation: Installation,
    /// The member who initiated a request, even when an owner approves later.
    pub requester: Option<Actor>,
    /// The owner or member who caused the event.
    pub sender: Actor,
}

/// Webhook signature verification before any JSON parsing or side effect.
#[cfg_attr(test, unimock::unimock(api = WebhookVerifierMock))]
#[async_trait]
pub trait WebhookVerifier: Send + Sync {
    /// Authenticate the exact request bytes with X-Hub-Signature-256.
    async fn verify(&self, signature: &str, body: &[u8]) -> Result<()>;
}

/// A SHA-256 GitHub webhook verifier with an opaque secret.
pub struct HmacVerifier {
    secret: SecretString,
}

impl HmacVerifier {
    /// Keep the configured webhook secret inside the control plane.
    pub fn new(secret: SecretString) -> Self {
        Self { secret }
    }
}

#[async_trait]
impl WebhookVerifier for HmacVerifier {
    async fn verify(&self, signature: &str, body: &[u8]) -> Result<()> {
        let Some(digest) = signature.strip_prefix("sha256=") else {
            return Err(Error::InvalidSignature);
        };
        let Ok(bytes) = hex::decode(digest) else {
            return Err(Error::InvalidSignature);
        };
        let mut mac =
            Hmac::<Sha256>::new_from_slice(self.secret.expose_secret().as_bytes()).internal()?;
        mac.update(body);
        if mac.verify_slice(&bytes).is_err() {
            return Err(Error::InvalidSignature);
        }
        Ok(())
    }
}
