//! Trusted clock and cryptographic entropy supplied at the composition root.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use ciproxy_store_interface::dashboard::records::CurrentCostClock;
use secrecy::SecretString;
use uuid::Uuid;

/// Adapt the existing service clock for a locked current-cost write.
pub(crate) struct CostClock(pub(crate) Arc<dyn Runtime>);
impl CurrentCostClock for CostClock {
    fn now(&self) -> DateTime<Utc> {
        self.0.now()
    }
}

use crate::error::{ErrorResultExt, Result};

/// Time and opaque identities used by dashboard services.
pub trait Runtime: Send + Sync {
    /// Return the current server UTC time.
    fn now(&self) -> DateTime<Utc>;
    /// Allocate a new stable opaque identity.
    fn identifier(&self) -> Uuid;
    /// Create an independent high-entropy browser or CSRF secret.
    fn secret(&self) -> Result<SecretString>;
}

/// Operating system clock and entropy for production composition.
pub struct SystemRuntime;

impl Runtime for SystemRuntime {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
    fn identifier(&self) -> Uuid {
        Uuid::new_v4()
    }
    fn secret(&self) -> Result<SecretString> {
        let mut bytes = [0; 32];
        getrandom::fill(&mut bytes).internal()?;
        Ok(bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
            .into())
    }
}
