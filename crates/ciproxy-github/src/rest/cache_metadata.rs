//! Complete repository cache metadata through the existing Actions read permission.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ciproxy_protocol::cache_metadata::CacheEntryMetadata;
use ciproxy_protocol::identifiers::GitRef;
use reqwest::Method;
use serde::Deserialize;

use crate::error::{Error, Result, ResultExt};

use super::http::RestClient;

/// Trusted repository entry identities, without archive contents or job scope input.
#[unimock::unimock(api = CacheEntriesMock)]
#[async_trait]
pub trait CacheEntries: Send + Sync {
    /// Read all pages of current metadata for one App-authorized repository.
    async fn cache_entries(
        &self,
        installation_id: i64,
        repository: &str,
    ) -> Result<Vec<CacheEntryMetadata>>;
}

#[derive(Deserialize)]
struct CachePage {
    actions_caches: Vec<RestEntry>,
}

#[derive(Deserialize)]
struct RestEntry {
    id: u64,
    #[serde(rename = "ref")]
    git_ref: GitRef,
    key: String,
    version: String,
    created_at: DateTime<Utc>,
}

#[async_trait]
impl CacheEntries for RestClient {
    async fn cache_entries(
        &self,
        installation_id: i64,
        repository: &str,
    ) -> Result<Vec<CacheEntryMetadata>> {
        let mut entries = Vec::new();
        for page in 1.. {
            let path = format!(
                "repos/{repository}/actions/caches?per_page=100&page={page}&sort=created_at&direction=asc"
            );
            let response = Self::success(
                self.request(installation_id, Method::GET, &path, &())
                    .await?,
            )?;
            let batch: CachePage = response.json().await.internal()?;
            let done = batch.actions_caches.len() < 100;
            for entry in batch.actions_caches {
                let created_at = match entry
                    .created_at
                    .timestamp_nanos_opt()
                    .and_then(|value| u64::try_from(value).ok())
                {
                    Some(value) => value.to_string(),
                    None => return Err(Error::AccessUnavailable(502)),
                };
                entries.push(CacheEntryMetadata {
                    id: entry.id.to_string(),
                    git_ref: entry.git_ref,
                    key: entry.key,
                    version: entry.version,
                    created_at,
                });
            }
            if done {
                return Ok(entries);
            }
        }
        Err(Error::AccessUnavailable(502))
    }
}
