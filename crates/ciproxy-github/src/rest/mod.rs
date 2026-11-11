//! Typed App REST boundary. It reads no job logs, artifacts, or code.

pub mod cache_metadata;
mod groups;
pub mod http;
mod job_times;

use async_trait::async_trait;
use ciproxy_store_interface::github::{Repository, RunFacts};

use crate::error::Result;

/// The GitHub API operations needed by installation and listener services.
#[cfg_attr(test, unimock::unimock(api = GitHubRestMock))]
#[async_trait]
pub trait GitHubRest: Send + Sync {
    /// Read real timing by exact run and unique ephemeral runner across all attempts.
    async fn job_times(
        &self,
        installation_id: i64,
        repository: &str,
        run_id: i64,
        runner_id: i64,
    ) -> Result<Option<ciproxy_store_interface::github_timing::JobTimes>>;
    /// Read every repository visible to one installation, with default branches.
    async fn repositories(&self, installation_id: i64) -> Result<Vec<Repository>>;
    /// Create or update an isolated group that allows exactly these repositories.
    async fn restrict_runner_group(
        &self,
        installation_id: i64,
        organization: &str,
        name: &str,
        repository_ids: &[i64],
        saved_id: Option<i64>,
    ) -> Result<i64>;
    /// Read trusted run metadata with the Actions read permission.
    async fn run_facts(
        &self,
        installation_id: i64,
        repository: &str,
        run_id: i64,
    ) -> Result<RunFacts>;
}
