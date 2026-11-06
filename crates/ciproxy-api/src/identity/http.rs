//! Redirect-free GitHub App OAuth and active membership REST adapter.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Duration;
use chrono::{DateTime, Utc};
use ciproxy_github::app::webhook::Account;
use ciproxy_github::installation_state::InstalledApp;
use ciproxy_store_interface::dashboard::records::Membership;
use reqwest::{Client, Url};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;

use crate::error::{Error, ErrorResultExt, Result};
use crate::runtime::Runtime;

use super::{Grant, Profile, UserIdentity};

/// Production GitHub App OAuth configuration. Debug never prints its secret.
pub struct OAuthConfig {
    /// Public App client identity.
    pub client_id: String,
    /// App client secret loaded from the operator's secret store.
    pub client_secret: SecretString,
    /// Exact configured HTTPS callback URL.
    pub callback_url: String,
    /// GitHub web origin, normally https://github.com/.
    pub github: Url,
    /// GitHub API origin, normally https://api.github.com/.
    pub api: Url,
}

/// OAuth adapter. The supplied client must disable redirects.
pub struct GithubOAuthClient {
    http: Client,
    config: OAuthConfig,
    runtime: Arc<dyn Runtime>,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: SecretString,
    expires_in: Option<i64>,
    refresh_token: Option<SecretString>,
    refresh_token_expires_in: Option<i64>,
}

#[derive(Deserialize)]
struct Organization {
    id: i64,
}
#[derive(Deserialize)]
struct OrganizationMembership {
    organization: Organization,
    role: String,
    state: String,
}

impl GithubOAuthClient {
    /// Construct at the hosted composition root with a redirect-free HTTP client.
    pub fn new(http: Client, config: OAuthConfig, runtime: Arc<dyn Runtime>) -> Self {
        Self {
            http,
            config,
            runtime,
        }
    }

    async fn token(&self, fields: &[(&str, &str)]) -> Result<Grant> {
        let response = self
            .http
            .post(
                self.config
                    .github
                    .join("login/oauth/access_token")
                    .internal()?,
            )
            .header("Accept", "application/json")
            .form(fields)
            .send()
            .await
            .internal()?;
        if !response.status().is_success() {
            return Err(Error::Unauthorized);
        }
        let response = match response.json::<TokenResponse>().await {
            Ok(value) => value,
            Err(_) => return Err(Error::Unauthorized),
        };
        let now = self.runtime.now();
        Ok(Grant {
            access_token: response.access_token,
            expires_at: response
                .expires_in
                .map(|seconds| now + Duration::seconds(seconds)),
            refresh_token: response.refresh_token,
            refresh_expires_at: response
                .refresh_token_expires_in
                .map(|seconds| now + Duration::seconds(seconds)),
        })
    }

    async fn get(&self, grant: &Grant, path: &str) -> Result<reqwest::Response> {
        let response = self
            .http
            .get(self.config.api.join(path).internal()?)
            .bearer_auth(grant.access_token.expose_secret())
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "ciproxy-api")
            .header("X-GitHub-Api-Version", "2026-03-10")
            .send()
            .await
            .internal()?;
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(Error::Unauthorized);
        }
        if !response.status().is_success() {
            return Err(Error::Unavailable);
        }
        Ok(response)
    }
}

#[async_trait]
impl UserIdentity for GithubOAuthClient {
    fn authorization_url(&self, state: &str, challenge: &str) -> String {
        let mut url = self.config.github.clone();
        url.set_path("login/oauth/authorize");
        url.query_pairs_mut()
            .append_pair("client_id", &self.config.client_id)
            .append_pair("redirect_uri", &self.config.callback_url)
            .append_pair("state", state)
            .append_pair("code_challenge", challenge)
            .append_pair("code_challenge_method", "S256")
            .append_pair("scope", "offline_access");
        url.to_string()
    }
    async fn exchange(&self, code: SecretString, verifier: SecretString) -> Result<Grant> {
        self.token(&[
            ("client_id", &self.config.client_id),
            ("client_secret", self.config.client_secret.expose_secret()),
            ("code", code.expose_secret()),
            ("redirect_uri", &self.config.callback_url),
            ("code_verifier", verifier.expose_secret()),
        ])
        .await
    }
    async fn refresh(&self, grant: Grant) -> Result<Grant> {
        let Some(refresh) = grant.refresh_token else {
            return Err(Error::Unauthorized);
        };
        if grant
            .refresh_expires_at
            .is_some_and(|at| self.runtime.now() >= at)
        {
            return Err(Error::Unauthorized);
        }
        self.token(&[
            ("client_id", &self.config.client_id),
            ("client_secret", self.config.client_secret.expose_secret()),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh.expose_secret()),
        ])
        .await
    }
    async fn profile(&self, grant: &Grant) -> Result<Profile> {
        self.get(grant, "user").await?.json().await.internal()
    }
    async fn installation(
        &self,
        grant: &Grant,
        installation_id: i64,
    ) -> Result<Option<InstalledApp>> {
        for page in 1u64.. {
            let batch: InstallationPage = self
                .get(
                    grant,
                    &format!("user/installations?per_page=100&page={page}"),
                )
                .await?
                .json()
                .await
                .internal()?;
            let done = batch.installations.len() < 100;
            if let Some(installation) = batch
                .installations
                .into_iter()
                .find(|installation| installation.id == installation_id)
            {
                return Ok(Some(InstalledApp {
                    id: installation.id,
                    account: installation.account,
                    suspended: installation.suspended_at.is_some(),
                }));
            }
            if done {
                return Ok(None);
            }
        }
        Err(Error::Unavailable)
    }
    async fn memberships(&self, grant: &Grant) -> Result<Vec<Membership>> {
        let mut result = Vec::new();
        for page in 1u64.. {
            let batch: Vec<OrganizationMembership> = self
                .get(
                    grant,
                    &format!("user/memberships/orgs?state=active&per_page=100&page={page}"),
                )
                .await?
                .json()
                .await
                .internal()?;
            let done = batch.len() < 100;
            result.extend(
                batch
                    .into_iter()
                    .filter(|member| member.state == "active")
                    .map(|member| Membership {
                        github_id: member.organization.id,
                        owner: member.role == "admin",
                    }),
            );
            if done {
                return Ok(result);
            }
        }
        Err(Error::Unavailable)
    }
}

#[derive(Deserialize)]
struct InstallationPage {
    installations: Vec<UserInstallation>,
}
#[derive(Deserialize)]
struct UserInstallation {
    id: i64,
    account: Account,
    suspended_at: Option<DateTime<Utc>>,
}
