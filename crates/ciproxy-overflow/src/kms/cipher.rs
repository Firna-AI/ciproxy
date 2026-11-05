//! Workspace-bound encryption with Cloud KMS CRC32C verification.

use std::sync::Arc;

use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::STANDARD};

use crate::crypto::{KeyCipher, KeyContext};
use crate::error::{Error, Result, ResultExt};
use crate::kms::transport::{KmsAction, KmsTransport};
use crate::secrets::SecretBytes;

use super::wire::{DecryptRequest, DecryptResponse, EncryptRequest, EncryptResponse};

/// Real Cloud KMS cipher over an authenticated HTTPS transport.
pub struct CloudKmsCipher {
    transport: Arc<dyn KmsTransport>,
}

impl CloudKmsCipher {
    /// Inject the selected KMS resource and OAuth transport at composition.
    pub fn new(transport: Arc<dyn KmsTransport>) -> Self {
        Self { transport }
    }
}

#[async_trait]
impl KeyCipher for CloudKmsCipher {
    async fn seal(&self, context: KeyContext, plaintext: SecretBytes) -> Result<Vec<u8>> {
        let aad = context.authenticated_data();
        let request = EncryptRequest {
            plaintext: STANDARD.encode(plaintext.expose()),
            additional_authenticated_data: STANDARD.encode(&aad),
            plaintext_crc32c: crc32c::crc32c(plaintext.expose()).to_string(),
            additional_authenticated_data_crc32c: crc32c::crc32c(&aad).to_string(),
        };
        let payload = SecretBytes::new(serde_json::to_vec(&request).internal()?);
        let response = self.transport.post(KmsAction::Encrypt, payload).await?;
        let response: EncryptResponse = serde_json::from_slice(response.expose()).internal()?;
        let bytes = STANDARD.decode(response.ciphertext).internal()?;
        if !response.verified_plaintext_crc32c
            || !response.verified_additional_authenticated_data_crc32c
            || response.ciphertext_crc32c.parse::<u32>().internal()? != crc32c::crc32c(&bytes)
            || !self.transport.owns_version(&response.name)
        {
            return Err(Error::KmsIntegrity);
        }
        Ok(bytes)
    }

    async fn open(&self, context: KeyContext, ciphertext: Vec<u8>) -> Result<SecretBytes> {
        let aad = context.authenticated_data();
        let request = DecryptRequest {
            ciphertext: STANDARD.encode(&ciphertext),
            additional_authenticated_data: STANDARD.encode(&aad),
            ciphertext_crc32c: crc32c::crc32c(&ciphertext).to_string(),
            additional_authenticated_data_crc32c: crc32c::crc32c(&aad).to_string(),
        };
        let payload = SecretBytes::new(serde_json::to_vec(&request).internal()?);
        let response = self.transport.post(KmsAction::Decrypt, payload).await?;
        let response: DecryptResponse = serde_json::from_slice(response.expose()).internal()?;
        let bytes = SecretBytes::new(STANDARD.decode(response.plaintext).internal()?);
        if response.plaintext_crc32c.parse::<u32>().internal()? != crc32c::crc32c(bytes.expose()) {
            return Err(Error::KmsIntegrity);
        }
        Ok(bytes)
    }
}
