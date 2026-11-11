//! Secret-free metadata work over the existing private cache admin listener.

use std::path::PathBuf;

use async_trait::async_trait;
use ciproxy_protocol::cache_metadata::{
    CacheMetadataAnswer, CacheMetadataRelayConfig, CacheMetadataWork,
};
use reqwest::{Client, Method, Response};
use serde::Serialize;

use super::error::{Error, ErrorResultExt, Result};

/// The root agent's authenticated local metadata administration boundary.
#[async_trait]
pub trait MetadataAdministration: Send + Sync {
    /// Enable requests without disclosing the machine credential or hosted URL.
    async fn enable(&self) -> Result<()>;
    /// Long-poll one queued job/request identity from the cache service.
    async fn next(&self) -> Result<Option<CacheMetadataWork>>;
    /// Return only metadata or an unavailable result for one pending request.
    async fn answer(&self, answer: CacheMetadataAnswer) -> Result<()>;
}

/// Local admin HTTP adapter. It holds only the existing cache admin token path.
pub struct HttpMetadataAdministration {
    client: Client,
    url: String,
    token_file: PathBuf,
}

impl HttpMetadataAdministration {
    /// Compose a private admin connection at a trusted host composition root.
    pub fn new(client: Client, url: String, token_file: PathBuf) -> Self {
        Self {
            client,
            url,
            token_file,
        }
    }

    async fn request(
        &self,
        method: Method,
        path: &str,
        body: Option<&impl Serialize>,
    ) -> Result<Response> {
        let token = tokio::fs::read_to_string(&self.token_file)
            .await
            .internal()?;
        let mut request = self
            .client
            .request(method, format!("{}{path}", self.url))
            .bearer_auth(token.trim());
        if let Some(body) = body {
            request = request.json(body);
        }
        let response = request.send().await.internal()?;
        if !response.status().is_success() {
            return Err(Error::Unavailable);
        }
        Ok(response)
    }
}

#[async_trait]
impl MetadataAdministration for HttpMetadataAdministration {
    async fn enable(&self) -> Result<()> {
        self.request(
            Method::PUT,
            "/metadata-relay",
            Some(&CacheMetadataRelayConfig { enabled: true }),
        )
        .await?;
        Ok(())
    }

    async fn next(&self) -> Result<Option<CacheMetadataWork>> {
        self.request(
            Method::GET,
            "/metadata-requests",
            None::<&CacheMetadataRelayConfig>,
        )
        .await?
        .json()
        .await
        .internal()
    }

    async fn answer(&self, answer: CacheMetadataAnswer) -> Result<()> {
        self.request(Method::POST, "/metadata-responses", Some(&answer))
            .await?;
        Ok(())
    }
}
