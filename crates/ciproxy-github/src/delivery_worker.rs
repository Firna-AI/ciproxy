//! Long-lived webhook recovery passes with one provider throttle deadline.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, TimeDelta, Utc};
use tokio::sync::Mutex;

use crate::error::{Error, Result};
use crate::redelivery::WebhookRecovery;
use crate::runtime::Runtime;

/// One supervised recovery pass, including provider throttle scheduling.
#[async_trait]
pub trait RecoveryPass: Send + Sync {
    /// Scan retained history unless the provider's throttle deadline is pending.
    async fn retry_failed(&self) -> Result<()>;
}

/// A recovery scheduler whose only retained state is the global throttle time.
pub struct RecoveryWorker {
    history: Arc<dyn WebhookRecovery>,
    runtime: Arc<dyn Runtime>,
    throttle_until: Mutex<Option<DateTime<Utc>>>,
}

impl RecoveryWorker {
    /// Bind one history scanner and its clock for the lifetime of the worker.
    pub fn new(history: Arc<dyn WebhookRecovery>, runtime: Arc<dyn Runtime>) -> Self {
        Self {
            history,
            runtime,
            throttle_until: Mutex::new(None),
        }
    }
}

#[async_trait]
impl RecoveryPass for RecoveryWorker {
    async fn retry_failed(&self) -> Result<()> {
        let mut throttle_until = self.throttle_until.lock().await;
        if throttle_until.is_some_and(|until| self.runtime.now() < until) {
            return Ok(());
        }
        *throttle_until = None;
        match self.history.retry_failed(self.runtime.as_ref()).await {
            Err(Error::DeliveryThrottled {
                retry_after_seconds,
                reset_at,
            }) => {
                let now = self.runtime.now();
                let until = if let Some(seconds) = retry_after_seconds {
                    TimeDelta::from_std(Duration::from_secs(seconds))
                        .ok()
                        .and_then(|delay| now.checked_add_signed(delay))
                        .unwrap_or(DateTime::<Utc>::MAX_UTC)
                } else {
                    reset_at.unwrap_or(now + TimeDelta::minutes(1))
                };
                *throttle_until = Some(until);
                tracing::warn!(%until, "GitHub webhook recovery waits for the provider rate limit");
                Ok(())
            }
            result => result,
        }
    }
}

#[cfg(test)]
#[path = "_tests_/delivery_worker_tests.rs"]
mod delivery_worker_tests;
