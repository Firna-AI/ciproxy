//! Agent-supplied identity and permissions for one job VM.

use ciproxy_protocol::identifiers::JobId;
use ciproxy_protocol::jobs::{JobFacts, JobScope};
use ciproxy_protocol::secret::Secret;
use serde::{Deserialize, Serialize};

/// Cache permissions obtained from the trusted runner assignment.
///
/// No permission is granted when the field is absent. Job-provided cache-mode
/// environment variables and request metadata never grant permission.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Permissions {
    /// The assigned job may read caches.
    #[serde(default)]
    pub read: bool,
    /// The assigned job may save caches within its trusted scope.
    #[serde(default)]
    pub write: bool,
}

/// Registration accepted only on the agent's authenticated admin listener.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Registration {
    /// Trusted GitHub job identifier.
    pub job_id: JobId,
    /// Facts read by the control plane from GitHub.
    pub facts: JobFacts,
    /// Effective permissions from the runner assignment, never job code.
    #[serde(default)]
    pub permissions: Permissions,
    /// Original GitHub runtime token for this assignment.
    #[serde(default = "missing_token")]
    pub github_token: Secret,
    /// Original ACTIONS_CACHE_URL for the legacy API.
    pub cache_url: Secret,
    /// Original ACTIONS_RESULTS_URL for v2 and results passthrough.
    pub results_url: Secret,
    /// Loopback URL of this VM's proxy, used for transfer URLs.
    pub proxy_url: String,
}

/// Opaque session credential issued by the machine to its agent.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SessionCredential {
    /// Random credential for this VM only.
    pub token: Secret,
    /// Machine-checked scope for the agent's Docker-disk policy.
    pub scope: JobScope,
}

fn missing_token() -> Secret {
    Secret::new("")
}
