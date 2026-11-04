//! Injected time and command identity boundaries.

use chrono::{DateTime, Utc};
use unimock::unimock;
use uuid::Uuid;

/// Time used for signatures, banner dates, and durable command creation.
#[unimock(api = ClockMock)]
pub trait Clock: Send + Sync {
    /// Read the current UTC instant.
    fn now(&self) -> DateTime<Utc>;
    /// Create a new server-owned mutation identifier.
    fn command_id(&self) -> Uuid;
}

/// The system clock used only by runtime composition.
#[derive(Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
    fn command_id(&self) -> Uuid {
        Uuid::new_v4()
    }
}
