//! App-JWT uninstall operation for intentional organization removal.

use std::sync::Arc;

use crate::wire::access::PendingInstallation;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ciproxy_github::app::webhook::{Account, AccountKind, Actor};
use ciproxy_github::auth::AppJwt;
use ciproxy_github::installation_state::InstalledApp;
use reqwest::{Client, StatusCode, Url};
use secrecy::ExposeSecret;
use serde::Deserialize;

use crate::error::{Error, ErrorResultExt, Result};

/// App-wide administrative operation used only after durable take-out.
#[async_trait]
pub trait InstallationAdministration: Send + Sync {
    /// Read the exact current installation authenticated as the configured App.
    /// This alone grants no association with a dashboard person.
    async fn installation(&self, installation_id: i64) -> Result<Option<InstalledApp>>;
    /// Read pending requests whose requester is this exact signed-in person.
    async fn pending_requests(&self, github_user_id: i64) -> Result<Vec<PendingInstallation>>;
    /// Check exact App access before retrying cleanup after a lost response.
    async fn exists(&self, installation_id: i64) -> Result<bool>;
    /// Remove exactly this installation. Already absent is successful.
    async fn uninstall(&self, installation_id: i64) -> Result<()>;
}

/// Redirect-free production adapter authenticated with the GitHub App JWT.
pub struct GithubInstallationAdmin {
    http: Client,
    api: Url,
    signer: Arc<dyn AppJwt>,
}

impl GithubInstallationAdmin {
    /// Construct at the composition root with the existing App JWT signer.
    pub fn new(http: Client, api: Url, signer: Arc<dyn AppJwt>) -> Self {
        Self { http, api, signer }
    }
}

#[async_trait]
impl InstallationAdministration for GithubInstallationAdmin {
    async fn installation(&self, installation_id: i64) -> Result<Option<InstalledApp>> {
        let response = self
            .get(&format!("app/installations/{installation_id}"))
            .await?;
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !response.status().is_success() {
            return Err(Error::Unavailable);
        }
        let installation: Installation = response.json().await.internal()?;
        if installation.id != installation_id {
            return Err(Error::Unavailable);
        }
        Ok(Some(InstalledApp {
            id: installation.id,
            account: installation.account,
            suspended: installation.suspended_at.is_some(),
        }))
    }
    async fn pending_requests(&self, github_user_id: i64) -> Result<Vec<PendingInstallation>> {
        let mut result = Vec::new();
        for page in 1u64.. {
            let response = self
                .get(&format!(
                    "app/installation-requests?per_page=100&page={page}"
                ))
                .await?;
            if !response.status().is_success() {
                return Err(Error::Unavailable);
            }
            let requests: Vec<InstallationRequest> = response.json().await.internal()?;
            let done = requests.len() < 100;
            result.extend(
                requests
                    .into_iter()
                    .filter(|request| {
                        request.requester.id == github_user_id
                            && request.account.kind == AccountKind::Organization
                    })
                    .map(|request| PendingInstallation {
                        id: request.id.to_string(),
                        github_id: request.account.id.to_string(),
                        organization: request.account.login,
                        created_at: request.created_at,
                    }),
            );
            if done {
                return Ok(result);
            }
        }
        Err(Error::Unavailable)
    }
    async fn exists(&self, installation_id: i64) -> Result<bool> {
        let response = self
            .get(&format!("app/installations/{installation_id}"))
            .await?;
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(false);
        }
        if !response.status().is_success() {
            return Err(Error::Unavailable);
        }
        let installation: Installation = response.json().await.internal()?;
        if installation.id != installation_id {
            return Err(Error::Unavailable);
        }
        Ok(true)
    }
    async fn uninstall(&self, installation_id: i64) -> Result<()> {
        let token = self.signer.token().internal()?;
        let response = self
            .http
            .delete(
                self.api
                    .join(&format!("app/installations/{installation_id}"))
                    .internal()?,
            )
            .bearer_auth(token.expose_secret())
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "ciproxy-api")
            .header("X-GitHub-Api-Version", "2026-03-10")
            .send()
            .await
            .internal()?;
        if response.status().is_success() || response.status() == StatusCode::NOT_FOUND {
            Ok(())
        } else {
            Err(Error::Unavailable)
        }
    }
}

#[derive(Deserialize)]
struct Installation {
    id: i64,
    account: Account,
    suspended_at: Option<DateTime<Utc>>,
}
#[derive(Deserialize)]
struct InstallationRequest {
    id: i64,
    account: Account,
    requester: Actor,
    created_at: DateTime<Utc>,
}

impl GithubInstallationAdmin {
    async fn get(&self, path: &str) -> Result<reqwest::Response> {
        let token = self.signer.token().internal()?;
        self.http
            .get(self.api.join(path).internal()?)
            .bearer_auth(token.expose_secret())
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "ciproxy-api")
            .header("X-GitHub-Api-Version", "2026-03-10")
            .send()
            .await
            .internal()
    }
}
