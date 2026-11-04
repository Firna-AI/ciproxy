//! Injected time and identity inputs for service orchestration.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Runtime inputs that unit tests supply without clocks or randomness.
#[cfg_attr(test, unimock::unimock(api = RuntimeMock))]
pub trait Runtime: Send + Sync {
    /// Read the current UTC time.
    fn now(&self) -> DateTime<Utc>;
    /// Allocate a new internal identity.
    fn uuid(&self) -> Uuid;
}

/// The process clock and UUID source, constructed at a composition root.
pub struct SystemRuntime;

impl Runtime for SystemRuntime {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
    fn uuid(&self) -> Uuid {
        Uuid::new_v4()
    }
}
