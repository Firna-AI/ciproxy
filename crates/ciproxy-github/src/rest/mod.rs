//! Typed App REST boundary. It reads no job logs, artifacts, or code.

mod groups;
pub mod http;

use async_trait::async_trait;
use ciproxy_store_interface::github::{Repository, RunFacts};

use crate::error::Result;

/// The GitHub API operations needed by installation and listener services.
#[cfg_attr(test, unimock::unimock(api = GitHubRestMock))]
#[async_trait]
pub trait GitHubRest: Send + Sync {
    /// Read every repository visible to one installation, with default branches.
    async fn repositories(&self, installation_id: i64) -> Result<Vec<Repository>>;
    /// Create or update an isolated group that allows exactly these repositories.
    async fn restrict_runner_group(
        &self,
        installation_id: i64,
        organization: &str,
        name: &str,
        repository_ids: &[i64],
    ) -> Result<i64>;
    /// Read trusted run metadata with the Actions read permission.
    async fn run_facts(
        &self,
        installation_id: i64,
        repository: &str,
        run_id: i64,
    ) -> Result<RunFacts>;
}
