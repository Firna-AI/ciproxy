//! Agent-supplied identity and permissions for one job VM.

use ciproxy_protocol::jobs::JobScope;
use ciproxy_protocol::secret::Secret;
use serde::{Deserialize, Serialize};

/// Opaque session credential issued by the machine to its agent.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SessionCredential {
    /// Random credential for this VM only.
    pub token: Secret,
    /// Machine-checked scope for the agent's Docker-disk policy.
    pub scope: JobScope,
}
