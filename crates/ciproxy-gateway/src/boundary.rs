//! Injectable authority, clock, entropy, and lifecycle event boundaries.

use chrono::{DateTime, Utc};
use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::secret::Secret;
use unimock::unimock;
use uuid::Uuid;

use crate::error::{ErrorResultExt, Result};
use crate::types::Session;

/// Trusted workspace authorization supplied by the sign-in service.
#[unimock(api = AuthorizationMock)]
pub trait Authorization: Send + Sync {
    /// Require the Admin role in this workspace, without a GitHub runner-right check.
    fn require_admin(&self, principal: Uuid, workspace_id: Uuid) -> Result<()>;
}

/// Server time, injected before any operation starts.
#[unimock(api = ClockMock)]
pub trait Clock: Send + Sync {
    /// Return the current UTC receipt or issuance time.
    fn now(&self) -> DateTime<Utc>;
}

/// Cryptographic secrets and opaque identities supplied by the composition root.
#[unimock(api = EntropyMock)]
pub trait Entropy: Send + Sync {
    /// Allocate an opaque identity.
    fn identifier(&self) -> Result<Uuid>;
    /// Generate an independent high-entropy secret.
    fn secret(&self) -> Result<Secret>;
}

/// Authenticated runner and command outcomes consumed by later services.
#[unimock(api = AgentEventsMock)]
pub trait AgentEvents: Send + Sync {
    /// Retire idle hosted registrations after persisted credential revocation.
    fn machine_removed(&self, workspace_id: Uuid, machine_id: Uuid) -> Result<()>;
    /// Handle a typed report within its authenticated machine scope.
    fn receive(&self, session: Session, message: AgentMessage) -> Result<()>;
}

/// Operating-system UTC clock for production composition.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Operating-system entropy for tokens and connection identifiers.
pub struct SystemEntropy;

impl Entropy for SystemEntropy {
    fn identifier(&self) -> Result<Uuid> {
        let mut bytes = [0; 16];
        getrandom::fill(&mut bytes).internal()?;
        Ok(Uuid::from_bytes(bytes))
    }

    fn secret(&self) -> Result<Secret> {
        let mut bytes = [0; 32];
        getrandom::fill(&mut bytes).internal()?;
        Ok(Secret::new(
            bytes
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>(),
        ))
    }
}
