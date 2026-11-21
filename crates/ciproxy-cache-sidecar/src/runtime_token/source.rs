//! Public-key fetches from a machine-owned endpoint, without job credentials.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use axum::body::{Body, to_bytes};
use axum::http::{HeaderMap, Method};
use jsonwebtoken::jwk::JwkSet;
use serde::Deserialize;

use crate::error::{CacheError, Result, ResultExt};
use crate::transport::{Outbound, Transport};

const GITHUB_JWKS: &str = "https://token.actions.githubusercontent.com/.well-known/jwks";

/// Machine-owned public-key configuration for isolated operator smoke checks.
#[derive(Deserialize)]
pub struct VerificationConfig {
    /// Trusted key source. Never accept this configuration from a job or VM.
    pub jwks_url: String,
}

/// Swappable key fetch boundary. It never receives a runtime token.
#[cfg_attr(test, unimock::unimock(api = KeySourceMock))]
#[async_trait]
pub trait KeySource: Send + Sync {
    /// Fetch the public signing-key set.
    async fn fetch(&self) -> Result<JwkSet>;
}

/// Bounded key fetch through the machine HTTP transport.
pub struct HttpKeys {
    network: Arc<dyn Transport>,
    url: String,
}

impl HttpKeys {
    /// Use the fixed production GitHub endpoint.
    pub fn github(network: Arc<dyn Transport>) -> Self {
        Self {
            network,
            url: GITHUB_JWKS.into(),
        }
    }

    /// Use an explicit machine-owned configuration for an offline smoke test.
    pub fn configured(network: Arc<dyn Transport>, config: VerificationConfig) -> Self {
        Self {
            network,
            url: config.jwks_url,
        }
    }
}

#[async_trait]
impl KeySource for HttpKeys {
    async fn fetch(&self) -> Result<JwkSet> {
        let request = self.network.send(Outbound {
            method: Method::GET,
            url: self.url.clone(),
            headers: HeaderMap::new(),
            body: Body::empty(),
        });
        let response = tokio::time::timeout(Duration::from_secs(10), request)
            .await
            .internal()??;
        if !response.status().is_success() {
            return Err(CacheError::Unavailable);
        }
        let bytes = tokio::time::timeout(
            Duration::from_secs(10),
            to_bytes(response.into_body(), 1024 * 1024),
        )
        .await
        .internal()?
        .internal()?;
        serde_json::from_slice(&bytes).internal()
    }
}
