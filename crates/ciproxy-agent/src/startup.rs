//! Daemon startup finishes terminal cleanup before it needs workload resources.

use std::sync::Arc;

use async_trait::async_trait;

use crate::boundaries::Images;
use crate::controller::Agent;
use crate::error::{Error, Result};
use crate::host_preparation::Preparation;
use crate::state::Identity;
use crate::terminal_completion::CompletionAction;

#[async_trait]
pub(crate) trait Startup: Send + Sync {
    /// Complete empty removal locally or prepare the daemon's saved jobs.
    async fn prepare(&self, identity: Option<Identity>, removing: bool) -> Result<StartupState>;
}

/// Startup either completes locally or provides the optional daemon identity.
#[derive(Debug)]
pub(crate) enum StartupState {
    Complete,
    Run(Option<Identity>),
}

pub(crate) struct AgentStartup {
    pub(crate) agent: Arc<dyn Agent>,
    pub(crate) preparation: Arc<dyn Preparation>,
    pub(crate) images: Arc<dyn Images>,
}

#[async_trait]
impl Startup for AgentStartup {
    async fn prepare(&self, identity: Option<Identity>, removing: bool) -> Result<StartupState> {
        if self.agent.complete_removal(CompletionAction::Stop).await? {
            return Ok(StartupState::Complete);
        }
        self.preparation.verify().await?;
        self.images.recover()?;
        if !removing && identity.is_none() {
            return Err(Error::NotInstalled);
        }
        Ok(StartupState::Run(identity))
    }
}

#[cfg(test)]
#[path = "_tests_/terminal_startup_tests.rs"]
mod terminal_startup_tests;
