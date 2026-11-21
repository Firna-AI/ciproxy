//! Bounded Cloud KMS REST requests and OAuth token acquisition.

use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use ciproxy_protocol::secret::Secret;
use reqwest::{Client, redirect::Policy};
use serde::Deserialize;
use unimock::unimock;

use crate::error::{Error, Result, ResultExt};
use crate::secrets::SecretBytes;

/// The two permitted Cloud KMS operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KmsAction {
    /// Symmetric authenticated encryption.
    Encrypt,
    /// Symmetric authenticated decryption.
    Decrypt,
}

/// OAuth tokens remain outside stored keys and sandbox input.
#[unimock(api = AccessTokensMock)]
#[async_trait]
pub trait AccessTokens: Send + Sync {
    /// Return a current token with the Cloud KMS or cloud-platform scope.
    async fn access_token(&self) -> Result<Secret>;
}

/// Object-safe Cloud KMS transport, also used by integrity unit tests.
#[unimock(api = KmsTransportMock)]
#[async_trait]
pub trait KmsTransport: Send + Sync {
    /// Send bounded secret JSON to the configured crypto key.
    async fn post(&self, action: KmsAction, payload: SecretBytes) -> Result<SecretBytes>;
    /// Confirm that the response names a version of the selected key.
    fn owns_version(&self, name: &str) -> bool;
}

/// Cloud KMS HTTPS transport with redirects disabled and bounded responses.
pub struct CloudKmsRest {
    resource: String,
    client: Client,
    tokens: Arc<dyn AccessTokens>,
}

impl CloudKmsRest {
    /// Select `projects/P/locations/L/keyRings/R/cryptoKeys/K` and runtime OAuth.
    pub fn new(resource: String, tokens: Arc<dyn AccessTokens>) -> Result<Self> {
        let segments: Vec<_> = resource.split('/').collect();
        if segments.len() != 8
            || segments[0] != "projects"
            || segments[2] != "locations"
            || segments[4] != "keyRings"
            || segments[6] != "cryptoKeys"
            || segments.iter().any(|part| {
                part.is_empty()
                    || !part
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
            })
        {
            return Err(Error::InvalidProviderData);
        }
        let client = Client::builder()
            .https_only(true)
            .redirect(Policy::none())
            .timeout(Duration::from_secs(30))
            .build()
            .internal()?;
        Ok(Self {
            resource,
            client,
            tokens,
        })
    }
}

#[async_trait]
impl KmsTransport for CloudKmsRest {
    async fn post(&self, action: KmsAction, payload: SecretBytes) -> Result<SecretBytes> {
        let action = match action {
            KmsAction::Encrypt => "encrypt",
            KmsAction::Decrypt => "decrypt",
        };
        let token = self.tokens.access_token().await?;
        let mut response = self
            .client
            .post(format!(
                "https://cloudkms.googleapis.com/v1/{}:{action}",
                self.resource
            ))
            .bearer_auth(token.expose_secret())
            .header("Content-Type", "application/json")
            .body(payload.expose().to_vec())
            .send()
            .await
            .internal()?;
        if !response.status().is_success() {
            return Err(Error::KmsHttp {
                status: response.status().as_u16(),
            });
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.internal()? {
            if chunk.len() > (256 * 1024usize).saturating_sub(bytes.len()) {
                return Err(Error::InvalidProviderData);
            }
            bytes.extend_from_slice(&chunk);
        }
        Ok(SecretBytes::new(bytes))
    }

    fn owns_version(&self, name: &str) -> bool {
        name.strip_prefix(&format!("{}/cryptoKeyVersions/", self.resource))
            .is_some_and(|version| {
                !version.is_empty() && version.bytes().all(|b| b.is_ascii_digit())
            })
    }
}

/// GKE Workload Identity access tokens from the Google metadata service.
pub struct MetadataAccessTokens {
    client: Client,
}

impl MetadataAccessTokens {
    /// Construct without requesting a token.
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: Client::builder()
                .redirect(Policy::none())
                .timeout(Duration::from_secs(10))
                .build()
                .internal()?,
        })
    }
}

#[derive(Deserialize)]
struct MetadataToken {
    access_token: Secret,
}

#[async_trait]
impl AccessTokens for MetadataAccessTokens {
    async fn access_token(&self) -> Result<Secret> {
        let response = self.client.get("http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token")
            .header("Metadata-Flavor", "Google").send().await.internal()?;
        let token: MetadataToken = response
            .error_for_status()
            .internal()?
            .json()
            .await
            .internal()?;
        Ok(token.access_token)
    }
}
