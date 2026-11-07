//! Typed host-only HTTP cache administration with private file credentials.

use std::path::PathBuf;
use std::time::Duration;

use async_trait::async_trait;
use ciproxy_protocol::cache::MachineCacheReport;
use ciproxy_protocol::jobs::JobFacts;
use ciproxy_protocol::jobs::JobScope;
use ciproxy_protocol::runtime::CacheRegistration;
use ciproxy_protocol::secret::Secret;
use reqwest::{Client, Method, Response};
use serde::{Deserialize, Serialize};

use crate::error::{Error, ErrorResultExt, Result};
use crate::process::{Commands, arguments};
use std::sync::Arc;

/// Opaque job session returned by authenticated machine administration.
#[derive(Clone, Debug, Deserialize)]
pub struct Credential {
    pub(crate) token: Secret,
    pub(crate) scope: JobScope,
}

#[cfg_attr(test, unimock::unimock(api = AdministrationMock))]
#[async_trait]
pub(crate) trait Administration: Send + Sync {
    async fn recover(&self, registration: &CacheRegistration) -> Result<Option<Credential>>;
    async fn fallback(&self, facts: &JobFacts) -> Result<()>;
    async fn budget(&self, bytes: u64) -> Result<()>;
    async fn report(&self) -> Result<MachineCacheReport>;
    async fn clear(&self) -> Result<()>;
    async fn register(&self, registration: CacheRegistration) -> Result<Credential>;
    async fn revoke(&self, facts: &JobFacts) -> Result<()>;
}

#[derive(Serialize)]
struct Budget {
    capacity_bytes: u64,
}

pub(crate) struct HttpAdministration {
    client: Client,
    token_file: PathBuf,
}

impl HttpAdministration {
    pub(crate) fn new(token_file: PathBuf) -> Result<Self> {
        Ok(Self {
            client: Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(Duration::from_secs(10))
                .build()
                .context("configure local cache client")?,
            token_file,
        })
    }

    async fn request(
        &self,
        method: Method,
        path: &str,
        body: Option<&impl Serialize>,
    ) -> Result<Response> {
        let token = tokio::fs::read_to_string(&self.token_file)
            .await
            .context("read private cache administration credential")?;
        let mut request = self
            .client
            .request(method, format!("http://127.0.0.1:8229{path}"))
            .bearer_auth(token.trim());
        if let Some(body) = body {
            request = request.json(body);
        }
        let response = request
            .send()
            .await
            .context("contact cache administration")?;
        if !response.status().is_success() {
            return Err(Error::CacheUnavailable);
        }
        Ok(response)
    }
}

#[async_trait]
impl Administration for HttpAdministration {
    async fn recover(&self, registration: &CacheRegistration) -> Result<Option<Credential>> {
        self.request(Method::POST, "/sessions/recover", Some(registration))
            .await?
            .json()
            .await
            .context("recover registered cache session")
    }
    async fn fallback(&self, facts: &JobFacts) -> Result<()> {
        self.request(Method::POST, "/sessions/fallback", Some(facts))
            .await?;
        Ok(())
    }
    async fn budget(&self, bytes: u64) -> Result<()> {
        self.request(
            Method::PUT,
            "/budget",
            Some(&Budget {
                capacity_bytes: bytes,
            }),
        )
        .await?;
        Ok(())
    }
    async fn report(&self) -> Result<MachineCacheReport> {
        self.request(Method::GET, "/report", None::<&Budget>)
            .await?
            .json()
            .await
            .context("decode cache report")
    }
    async fn clear(&self) -> Result<()> {
        self.request(Method::POST, "/clear", None::<&Budget>)
            .await?;
        Ok(())
    }
    async fn register(&self, registration: CacheRegistration) -> Result<Credential> {
        self.request(Method::POST, "/sessions", Some(&registration))
            .await?
            .json()
            .await
            .context("decode checked cache session")
    }
    async fn revoke(&self, facts: &JobFacts) -> Result<()> {
        self.request(Method::POST, "/sessions/close", Some(facts))
            .await?;
        Ok(())
    }
}

#[cfg_attr(test, unimock::unimock(api = CacheFilesMock))]
#[async_trait]
pub(crate) trait CacheFiles: Send + Sync {
    async fn release_occupied_space(&self) -> Result<()>;
}

pub(crate) struct OwnedCacheFiles {
    pub(crate) directory: PathBuf,
    pub(crate) commands: Arc<dyn Commands>,
}

#[async_trait]
impl CacheFiles for OwnedCacheFiles {
    async fn release_occupied_space(&self) -> Result<()> {
        for directory in [self.directory.clone(), self.directory.join("data")] {
            let mut files = match tokio::fs::read_dir(&directory).await {
                Ok(files) => files,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Err(error).context("inspect unavailable cache data"),
            };
            if files
                .next_entry()
                .await
                .context("inspect occupied cache data")?
                .is_none()
            {
                continue;
            }
            self.commands
                .run(
                    "/usr/local/bin/ciproxy-cache-sidecar",
                    &arguments(&["reclaim", "--cache-dir", &directory.to_string_lossy()]),
                    &[],
                )
                .await?;
        }
        Ok(())
    }
}
