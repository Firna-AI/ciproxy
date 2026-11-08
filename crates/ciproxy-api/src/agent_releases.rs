//! Trusted latest agent metadata from the same release host as the installer.

use std::time::Duration;

use async_trait::async_trait;
use reqwest::{Client, redirect};
use serde::Deserialize;
use unimock::unimock;

use crate::error::{Error, ErrorResultExt, Result};

/// Current supported agent release supplied by the configured release host.
#[unimock(api = AgentReleasesMock)]
#[async_trait]
pub trait AgentReleases: Send + Sync {
    /// Fetch the newest published version for the supported Linux target.
    async fn latest(&self) -> Result<String>;
}

/// Redirect-free bounded HTTP adapter for the configured trusted release host.
pub struct AgentReleaseClient {
    base: String,
    http: std::result::Result<Client, reqwest::Error>,
}

#[derive(Deserialize)]
struct Release {
    binary: String,
    version: String,
    target: String,
}

impl AgentReleaseClient {
    /// Construct the production HTTPS-only adapter without changing API callers.
    pub fn new(base: String) -> Self {
        Self {
            base,
            http: Client::builder()
                .https_only(true)
                .redirect(redirect::Policy::none())
                .connect_timeout(Duration::from_secs(5))
                .timeout(Duration::from_secs(10))
                .build(),
        }
    }

    /// Supply a redirect-free transport at composition, including a local test peer.
    pub fn with_client(base: String, http: Client) -> Self {
        Self {
            base,
            http: Ok(http),
        }
    }
}

#[async_trait]
impl AgentReleases for AgentReleaseClient {
    async fn latest(&self) -> Result<String> {
        let Ok(http) = &self.http else {
            return Err(Error::Unavailable);
        };
        let response = http
            .get(format!(
                "{}/releases/latest/ciproxy-agent/x86_64-unknown-linux-gnu",
                self.base.trim_end_matches('/')
            ))
            .send()
            .await
            .internal()?;
        if !response.status().is_success() {
            return Err(Error::Unavailable);
        }
        let release: Release = response.json().await.internal()?;
        if release.binary != "ciproxy-agent"
            || release.target != "x86_64-unknown-linux-gnu"
            || release.version.is_empty()
        {
            return Err(Error::Unavailable);
        }
        Ok(release.version)
    }
}
