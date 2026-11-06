//! Identity-bound encryption for retained OAuth grants and PKCE verifiers.

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use async_trait::async_trait;
use secrecy::{ExposeSecret, SecretString};
use uuid::Uuid;

use crate::error::{Error, ErrorResultExt, Result};

/// Encryption boundary. The persistence layer receives only ciphertext.
#[async_trait]
pub trait CredentialCipher: Send + Sync {
    /// Seal a grant or verifier for the exact supplied identity.
    async fn seal(&self, id: Uuid, value: SecretString) -> Result<Vec<u8>>;
    /// Open ciphertext only for its original identity.
    async fn open(&self, id: Uuid, value: Vec<u8>) -> Result<SecretString>;
}

/// AES-256-GCM adapter for a key loaded from the operator's secret store.
/// The caller owns key retrieval and rotation. Debug is intentionally absent.
pub struct AeadCredentialCipher {
    cipher: Aes256Gcm,
}

impl AeadCredentialCipher {
    /// Construct from a 32-byte encryption key at the composition root.
    pub fn new(key: &[u8; 32]) -> Self {
        Self {
            cipher: Aes256Gcm::new(key.into()),
        }
    }
}

#[async_trait]
impl CredentialCipher for AeadCredentialCipher {
    async fn seal(&self, id: Uuid, value: SecretString) -> Result<Vec<u8>> {
        let mut nonce = [0; 12];
        getrandom::fill(&mut nonce).internal()?;
        let payload = Payload {
            msg: value.expose_secret().as_bytes(),
            aad: id.as_bytes(),
        };
        let ciphertext = match self.cipher.encrypt(&Nonce::from(nonce), payload) {
            Ok(value) => value,
            Err(_) => return Err(Error::Unavailable),
        };
        Ok([nonce.to_vec(), ciphertext].concat())
    }
    async fn open(&self, id: Uuid, value: Vec<u8>) -> Result<SecretString> {
        let Some((nonce, ciphertext)) = value.split_at_checked(12) else {
            return Err(Error::Unavailable);
        };
        let nonce: [u8; 12] = match nonce.try_into() {
            Ok(value) => value,
            Err(_) => return Err(Error::Unavailable),
        };
        let bytes = match self.cipher.decrypt(
            &Nonce::from(nonce),
            Payload {
                msg: ciphertext,
                aad: id.as_bytes(),
            },
        ) {
            Ok(value) => value,
            Err(_) => return Err(Error::Unavailable),
        };
        Ok(String::from_utf8(bytes).internal()?.into())
    }
}
