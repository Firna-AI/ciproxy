//! Injected time and identity inputs for service orchestration.

use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Clock, pause, and identity inputs that tests control at their boundary.
#[unimock::unimock(api = RuntimeMock)]
#[async_trait]
pub trait Runtime: Send + Sync {
    /// Read the current UTC time.
    fn now(&self) -> DateTime<Utc>;
    /// Allocate a new internal identity.
    fn uuid(&self) -> Uuid;
    /// Pause between provider mutations without blocking an async worker.
    async fn pause(&self, duration: Duration) {
        tokio::time::sleep(duration).await;
    }
}

/// The process clock, pause, and UUID source, built at a composition root.
pub struct SystemRuntime;

impl Runtime for SystemRuntime {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
    fn uuid(&self) -> Uuid {
        Uuid::new_v4()
    }
}
