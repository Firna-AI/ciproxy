//! Host-only trusted upstream cache identities, separate from guest registration.

use serde::{Deserialize, Serialize};

use crate::identifiers::{CacheMetadataRequestId, GitRef, JobId};
use crate::secret::Secret;

/// Hosted metadata route on the existing agent gateway origin.
pub const CACHE_METADATA_PATH: &str = "/agent/cache-metadata";

/// Root-only hosted connection. Never send it to the cache service or a guest.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CacheMetadataSource {
    /// Hosted endpoint derived from the agent gateway origin.
    pub url: Secret,
    /// Existing machine credential. Never deliver this value to a guest.
    pub credential: Secret,
}

/// Exact hosted assignment whose repository metadata the machine requests.
#[derive(Deserialize, Serialize)]
pub struct CacheMetadataRequest {
    /// Assigned scale-set job identity. Its placement is checked by the host.
    pub job_id: JobId,
}

/// Trusted identity of one upstream entry, independent of local visibility.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CacheEntryMetadata {
    /// Stable GitHub cache identifier, represented as a decimal string.
    pub id: String,
    /// Full source ref returned by the GitHub App REST API.
    pub git_ref: GitRef,
    /// Original cache key.
    pub key: String,
    /// Original archive version.
    pub version: String,
    /// Upstream creation time as decimal-string Unix nanoseconds.
    pub created_at: String,
}

/// Complete current repository listing from the hosted GitHub App boundary.
#[derive(Clone, Deserialize, Serialize)]
pub struct CacheMetadataResponse {
    /// Every upstream entry visible to the installation in this repository.
    pub entries: Vec<CacheEntryMetadata>,
}

/// Secret-free administration flag that enables current metadata requests.
#[derive(Deserialize, Serialize)]
pub struct CacheMetadataRelayConfig {
    /// Whether the root agent will answer cache metadata work.
    pub enabled: bool,
}

/// One restore-time metadata operation queued for the root agent.
#[derive(Clone, Deserialize, Serialize)]
pub struct CacheMetadataWork {
    /// Opaque identifier used exactly once for its response.
    pub request_id: CacheMetadataRequestId,
    /// Exact assigned job whose repository the hosted route authorizes.
    pub job_id: JobId,
}

/// A root agent answer containing metadata or an unavailable result, never secrets.
#[derive(Deserialize, Serialize)]
pub struct CacheMetadataAnswer {
    /// The one pending request to complete.
    pub request_id: CacheMetadataRequestId,
    /// Current metadata, or no value when the trusted source is unavailable.
    pub metadata: Option<CacheMetadataResponse>,
}
