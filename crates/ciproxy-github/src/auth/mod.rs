//! GitHub App JWT signing and rotating installation tokens.

pub mod jwt;
pub mod tokens;

use async_trait::async_trait;
use secrecy::{ExposeSecret, SecretString};

use crate::error::Result;

/// A signer for short-lived GitHub App authentication.
#[cfg_attr(test, unimock::unimock(api = AppJwtMock))]
pub trait AppJwt: Send + Sync {
    /// Produce a newly signed App JWT without exposing it to diagnostics.
    fn token(&self) -> Result<SecretString>;
}

/// Installation authentication used by the REST and scale-set adapters.
#[cfg_attr(test, unimock::unimock(api = InstallationTokensMock))]
#[async_trait]
pub trait InstallationTokens: Send + Sync {
    /// Read a cached token, or issue a new one before expiry.
    async fn token(&self, installation_id: i64) -> Result<SecretString>;
    /// Discard a token after GitHub rejects it.
    async fn invalidate(&self, installation_id: i64);
}

/// An App JWT bridge for the scale-set SDK's credential provider.
pub struct ScaleSetJwt {
    /// The same signer used by the REST authentication adapter.
    pub signer: std::sync::Arc<dyn AppJwt>,
}

impl scaleset::JwtProvider for ScaleSetJwt {
    async fn token(&self) -> std::result::Result<String, scaleset::BoxError> {
        Ok(self.signer.token()?.expose_secret().to_owned())
    }
}
