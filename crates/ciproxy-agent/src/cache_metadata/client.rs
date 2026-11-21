//! Root-only hosted metadata client with the existing machine credential.

use async_trait::async_trait;
use ciproxy_protocol::cache_metadata::{
    CacheMetadataRequest, CacheMetadataResponse, CacheMetadataSource,
};
use ciproxy_protocol::identifiers::JobId;
use reqwest::Client;

use super::error::{Error, ErrorResultExt, Result};

/// Current repository metadata for a machine-authorized assignment.
#[async_trait]
pub trait TrustedMetadata: Send + Sync {
    /// Ask the hosted route to authenticate and resolve the exact job placement.
    async fn lookup(&self, job_id: JobId) -> Result<CacheMetadataResponse>;
}

/// This adapter and its credential must remain in the root agent process.
pub struct RootMetadataClient {
    client: Client,
    source: CacheMetadataSource,
}

impl RootMetadataClient {
    /// Compose the root-only connection with an HTTP client that disables redirects.
    pub fn new(client: Client, source: CacheMetadataSource) -> Self {
        Self { client, source }
    }
}

#[async_trait]
impl TrustedMetadata for RootMetadataClient {
    async fn lookup(&self, job_id: JobId) -> Result<CacheMetadataResponse> {
        let response = self
            .client
            .post(self.source.url.expose_secret())
            .bearer_auth(self.source.credential.expose_secret())
            .json(&CacheMetadataRequest { job_id })
            .send()
            .await
            .internal()?;
        if !response.status().is_success() {
            return Err(Error::Unavailable);
        }
        response.json().await.internal()
    }
}
