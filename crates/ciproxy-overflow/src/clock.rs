//! Injected UTC time for lifetime, catalog, retry, and spend boundaries.

use chrono::{DateTime, Utc};
use unimock::unimock;

/// UTC time sampled at each externally observable operation.
#[unimock(api = ClockMock)]
pub trait Clock: Send + Sync {
    /// Read the current UTC instant.
    fn now(&self) -> DateTime<Utc>;
}

/// The production UTC wall clock.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
