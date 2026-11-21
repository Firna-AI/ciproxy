//! Narrow workspace-bound encryption for Boat keys and runner launch input.

use async_trait::async_trait;
use unimock::unimock;
use uuid::Uuid;

use crate::error::Result;
use crate::secrets::SecretBytes;

/// The kind of sealed secret, included in KMS authenticated data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecretPurpose {
    /// One account API key version.
    BoatKey,
    /// One runner registration and callback credential.
    Launch,
}

/// Immutable authority authenticated with every encrypted secret.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeyContext {
    /// Workspace that alone may use this secret.
    pub workspace_id: Uuid,
    /// Key version or delivery identifier.
    pub id: Uuid,
    /// Prevents one secret type from being opened as another.
    pub purpose: SecretPurpose,
}

impl KeyContext {
    pub(crate) fn authenticated_data(self) -> Vec<u8> {
        let purpose = match self.purpose {
            SecretPurpose::BoatKey => "boat-key",
            SecretPurpose::Launch => "launch",
        };
        format!(
            "ciproxy-overflow/v1/{}/{}/{purpose}",
            self.workspace_id, self.id
        )
        .into_bytes()
    }
}

/// Only ciphertext crosses the persistence boundary.
#[unimock(api = KeyCipherMock)]
#[async_trait]
pub trait KeyCipher: Send + Sync {
    /// Encrypt bytes and bind them to this workspace and immutable identity.
    async fn seal(&self, context: KeyContext, plaintext: SecretBytes) -> Result<Vec<u8>>;
    /// Decrypt only when the authenticated workspace and identity match.
    async fn open(&self, context: KeyContext, ciphertext: Vec<u8>) -> Result<SecretBytes>;
}
