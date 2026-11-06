//! Original runner assignment data, carried only through trusted local channels.

use serde::{Deserialize, Serialize};

use crate::cache::CacheConnection;
use crate::identifiers::JobId;
use crate::jobs::JobFacts;
use crate::secret::Secret;

/// Effective cache rights from GitHub's assignment, before job environment setup.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct CachePermissions {
    /// The assignment permits restores.
    pub read: bool,
    /// The assignment permits saves.
    pub write: bool,
}

/// Original credentials extracted from the runner's server assignment.
/// These values must never enter hosted messages, logs, or durable agent state.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RuntimeCredentials {
    /// Preloaded Docker bytes measured by the trusted guest before action code.
    #[serde(default)]
    pub docker_data_bytes: u64,
    /// Proxy URL derived by the trusted guest from Docker's actual bridge.
    #[serde(default)]
    pub proxy_url: String,
    /// Original ACTIONS_RUNTIME_TOKEN.
    pub github_token: Secret,
    /// Original cache v1 endpoint.
    pub cache_url: Secret,
    /// Original cache v2 and results endpoint.
    pub results_url: Secret,
    /// Effective assignment rights, never job environment values.
    pub permissions: CachePermissions,
}

/// Authenticated host registration for one job's cache session.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CacheRegistration {
    /// The scale-message job UUID.
    pub job_id: JobId,
    /// Hosted API facts and expected token identities.
    pub facts: JobFacts,
    /// Original effective assignment rights.
    #[serde(default)]
    pub permissions: CachePermissions,
    /// Original runtime token, retained in memory only.
    #[serde(default = "missing_token")]
    pub github_token: Secret,
    /// Original cache v1 endpoint.
    pub cache_url: Secret,
    /// Original results endpoint.
    pub results_url: Secret,
    /// VM-local proxy address reachable from Docker containers.
    pub proxy_url: String,
}

/// In-memory configuration for the disposable guest proxy.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CacheProxyConfig {
    /// Machine cache listener reachable from this VM.
    pub machine_url: Secret,
    /// Narrow session credential, never the cache administration credential.
    pub machine_token: Secret,
    /// Original credentials and hosted facts for results passthrough.
    pub registration: CacheRegistration,
}

/// One trusted parent-pipe update, correlated without echoing a credential.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CacheProxyUpdate {
    /// Monotonic acknowledgement identifier within this proxy process.
    pub sequence: u64,
    /// Replacement machine endpoint and narrow session credential.
    pub connection: CacheConnection,
}

fn missing_token() -> Secret {
    Secret::new("")
}
