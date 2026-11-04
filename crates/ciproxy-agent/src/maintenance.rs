//! Periodic maintenance leaves VM units alone and logs recoverable failures.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::time::sleep;

use crate::boundaries::{Delay, Updates};
use crate::error::Result;

pub(crate) struct TokioDelay;

#[async_trait]
impl Delay for TokioDelay {
    async fn wait(&self, seconds: u64) {
        sleep(Duration::from_secs(seconds)).await;
    }
}

#[async_trait]
pub(crate) trait Maintenance: Send + Sync {
    async fn cycle(&self) -> Result<()>;
    async fn run(&self);
}

pub(crate) struct HourlyMaintenance {
    pub(crate) updates: Arc<dyn Updates>,
    pub(crate) delay: Arc<dyn Delay>,
}

#[async_trait]
impl Maintenance for HourlyMaintenance {
    async fn cycle(&self) -> Result<()> {
        if self.updates.check().await? {
            self.updates.restart().await?;
        }
        Ok(())
    }

    async fn run(&self) {
        loop {
            self.delay.wait(3600).await;
            if let Err(error) = self.cycle().await {
                tracing::warn!(error = %error, "agent update failed; keep current agent and VMs");
            }
        }
    }
}

#[cfg(test)]
#[path = "_tests_/maintenance_tests.rs"]
mod maintenance_tests;
