//! Secret-safe installation token caching and refresh.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use reqwest::{Client, StatusCode, Url};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::error::{Error, Result, ResultExt};
use crate::runtime::Runtime;

use super::{AppJwt, InstallationTokens};

/// A rotating installation-token adapter with one cache per App process.
pub struct RotatingTokens {
    http: Client,
    api: Url,
    signer: Arc<dyn AppJwt>,
    runtime: Arc<dyn Runtime>,
    cache: Mutex<HashMap<i64, Token>>,
}

#[derive(Deserialize)]
struct Token {
    token: SecretString,
    expires_at: DateTime<Utc>,
}

#[derive(Serialize)]
struct TokenRequest {
    permissions: Permissions,
}

#[derive(Serialize)]
struct Permissions {
    organization_self_hosted_runners: &'static str,
    metadata: &'static str,
    actions: &'static str,
}

impl RotatingTokens {
    /// Construct the adapter with a non-redirecting HTTP client.
    pub fn new(http: Client, api: Url, signer: Arc<dyn AppJwt>, runtime: Arc<dyn Runtime>) -> Self {
        Self {
            http,
            api,
            signer,
            runtime,
            cache: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl InstallationTokens for RotatingTokens {
    async fn token(&self, installation_id: i64) -> Result<SecretString> {
        let mut cache = self.cache.lock().await;
        if let Some(token) = cache.get(&installation_id)
            && token.expires_at > self.runtime.now() + Duration::seconds(60)
        {
            return Ok(token.token.clone());
        }
        let jwt = self.signer.token()?;
        let url = self
            .api
            .join(&format!(
                "app/installations/{installation_id}/access_tokens"
            ))
            .internal()?;
        let response = self
            .http
            .post(url)
            .bearer_auth(jwt.expose_secret())
            .header("Accept", "application/vnd.github+json")
            .header(
                "User-Agent",
                concat!("ciproxy-github/", env!("CARGO_PKG_VERSION")),
            )
            .header("X-GitHub-Api-Version", "2026-03-10")
            .json(&TokenRequest {
                permissions: Permissions {
                    organization_self_hosted_runners: "write",
                    metadata: "read",
                    actions: "read",
                },
            })
            .send()
            .await
            .internal()?;
        if response.status() == StatusCode::NOT_FOUND {
            return Err(Error::InstallationPending);
        }
        if !response.status().is_success() {
            return Err(Error::AccessUnavailable(response.status().as_u16()));
        }
        let token: Token = response.json().await.internal()?;
        let secret = token.token.clone();
        cache.insert(installation_id, token);
        Ok(secret)
    }

    async fn invalidate(&self, installation_id: i64) {
        self.cache.lock().await.remove(&installation_id);
    }
}
