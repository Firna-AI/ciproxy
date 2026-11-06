//! Authorized REST requests and complete installation repository pagination.

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_store_interface::github::{Repository, RunFacts};
use reqwest::{Client, Method, Response, StatusCode, Url};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};

use crate::auth::InstallationTokens;
use crate::error::{Error, Result, ResultExt};

use super::GitHubRest;

/// The GitHub REST adapter. Its HTTP client must disable redirects.
pub struct RestClient {
    pub(super) http: Client,
    pub(super) api: Url,
    tokens: Arc<dyn InstallationTokens>,
}

#[derive(Deserialize)]
struct RepositoryPage {
    repositories: Vec<Repository>,
}

impl RestClient {
    /// Build the adapter at a composition root with its token boundary.
    pub fn new(http: Client, api: Url, tokens: Arc<dyn InstallationTokens>) -> Self {
        Self { http, api, tokens }
    }

    pub(super) async fn request(
        &self,
        installation_id: i64,
        method: Method,
        path: &str,
        body: &impl Serialize,
    ) -> Result<Response> {
        let url = self.api.join(path).internal()?;
        for attempt in 0..2 {
            let token = self.tokens.token(installation_id).await?;
            let mut request = self
                .http
                .request(method.clone(), url.clone())
                .bearer_auth(token.expose_secret())
                .header("Accept", "application/vnd.github+json")
                .header(
                    "User-Agent",
                    concat!("ciproxy-github/", env!("CARGO_PKG_VERSION")),
                )
                .header("X-GitHub-Api-Version", "2026-03-10");
            if method != Method::GET {
                request = request.json(body);
            }
            let response = request.send().await.internal()?;
            if response.status() == StatusCode::UNAUTHORIZED && attempt == 0 {
                self.tokens.invalidate(installation_id).await;
                continue;
            }
            return Ok(response);
        }
        Err(Error::AccessUnavailable(401))
    }

    pub(super) fn success(response: Response) -> Result<Response> {
        if response.status().is_success() {
            Ok(response)
        } else {
            Err(Error::AccessUnavailable(response.status().as_u16()))
        }
    }
}

#[async_trait]
impl GitHubRest for RestClient {
    async fn job_times(
        &self,
        installation_id: i64,
        repository: &str,
        run_id: i64,
        runner_id: i64,
    ) -> Result<Option<ciproxy_store_interface::github_timing::JobTimes>> {
        self.times(installation_id, repository, run_id, runner_id)
            .await
    }
    async fn repositories(&self, installation_id: i64) -> Result<Vec<Repository>> {
        let mut result = Vec::new();
        let mut page = 1;
        loop {
            let path = format!("installation/repositories?per_page=100&page={page}");
            let response = Self::success(
                self.request(installation_id, Method::GET, &path, &())
                    .await?,
            )?;
            let batch: RepositoryPage = response.json().await.internal()?;
            let done = batch.repositories.len() < 100;
            result.extend(batch.repositories);
            if done {
                return Ok(result);
            }
            page += 1;
        }
    }

    async fn restrict_runner_group(
        &self,
        installation_id: i64,
        organization: &str,
        name: &str,
        repository_ids: &[i64],
        saved_id: Option<i64>,
    ) -> Result<i64> {
        self.sync_group(
            installation_id,
            organization,
            name,
            repository_ids,
            saved_id,
        )
        .await
    }

    async fn run_facts(
        &self,
        installation_id: i64,
        repository: &str,
        run_id: i64,
    ) -> Result<RunFacts> {
        let path = format!("repos/{repository}/actions/runs/{run_id}");
        let response = Self::success(
            self.request(installation_id, Method::GET, &path, &())
                .await?,
        )?;
        response.json().await.internal()
    }
}
