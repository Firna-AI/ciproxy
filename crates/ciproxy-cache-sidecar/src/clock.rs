//! Injected time for eviction and the rolling report window.

use std::time::{SystemTime, UNIX_EPOCH};

/// Time source at the machine boundary.
pub trait Clock: Send + Sync {
    /// Unix time in seconds.
    fn now(&self) -> u64;
}

/// The machine's system clock.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}
