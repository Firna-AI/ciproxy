//! Authoritative installation identity and suspension, authenticated as the App.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::{Client, StatusCode, Url};
use secrecy::ExposeSecret;
use serde::Deserialize;

use crate::app::webhook::Account;
use crate::auth::AppJwt;
use crate::error::{Error, Result, ResultExt};

/// The current installed App identity and state, independent of event order.
#[derive(Clone, Debug)]
pub struct InstalledApp {
    /// The installation that GitHub names now.
    pub id: i64,
    /// The owning GitHub account and current login.
    pub account: Account,
    /// Whether GitHub currently suspends the installation.
    pub suspended: bool,
}

/// App-JWT lookups that require no additional installation permission.
#[unimock::unimock(api = InstallationApiMock)]
#[async_trait]
pub trait InstallationApi: Send + Sync {
    /// Resolve the account identity before acquiring its organization lease.
    /// InstallationPending means an exact App-JWT installation endpoint returned 404.
    async fn by_id(&self, installation_id: i64) -> Result<InstalledApp>;
    /// Read the current installation while the organization lease is held.
    async fn for_organization(&self, login: &str) -> Result<InstalledApp>;
}

/// The App-authenticated REST implementation, constructed at the composition root.
pub struct AppInstallationClient {
    http: Client,
    api: Url,
    signer: Arc<dyn AppJwt>,
}

#[derive(Deserialize)]
struct InstallationResponse {
    id: i64,
    account: Account,
    suspended_at: Option<DateTime<Utc>>,
}

impl AppInstallationClient {
    /// Use the control-plane signer and a client that disables redirects.
    pub fn new(http: Client, api: Url, signer: Arc<dyn AppJwt>) -> Self {
        Self { http, api, signer }
    }

    async fn lookup(&self, path: &str) -> Result<InstalledApp> {
        let jwt = self.signer.token()?;
        let response = self
            .http
            .get(self.api.join(path).internal()?)
            .bearer_auth(jwt.expose_secret())
            .header("Accept", "application/vnd.github+json")
            .header(
                "User-Agent",
                concat!("ciproxy-github/", env!("CARGO_PKG_VERSION")),
            )
            .header("X-GitHub-Api-Version", "2026-03-10")
            .send()
            .await
            .internal()?;
        if response.status() == StatusCode::NOT_FOUND {
            return Err(Error::InstallationPending);
        }
        if !response.status().is_success() {
            return Err(Error::AccessUnavailable(response.status().as_u16()));
        }
        let snapshot: InstallationResponse = response.json().await.internal()?;
        Ok(InstalledApp {
            id: snapshot.id,
            account: snapshot.account,
            suspended: snapshot.suspended_at.is_some(),
        })
    }
}

#[async_trait]
impl InstallationApi for AppInstallationClient {
    async fn by_id(&self, installation_id: i64) -> Result<InstalledApp> {
        self.lookup(&format!("app/installations/{installation_id}"))
            .await
    }
    async fn for_organization(&self, login: &str) -> Result<InstalledApp> {
        self.lookup(&format!("orgs/{login}/installation")).await
    }
}
