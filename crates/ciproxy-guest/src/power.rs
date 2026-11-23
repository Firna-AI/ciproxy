//! End an own-machine VM or leave sandbox cleanup to its provider.

use async_trait::async_trait;
use tokio::process::Command;

use crate::boundary::Power;
use crate::error::{Error, ErrorResultExt, Result};
use crate::platform::Platform;

#[cfg_attr(test, unimock::unimock(api = ShutdownCommandMock))]
#[async_trait]
pub(crate) trait ShutdownCommand: Send {
    async fn run(&mut self, program: &str, arguments: &[&str]) -> Result<bool>;
}

pub(crate) struct SystemShutdownCommand;

#[async_trait]
impl ShutdownCommand for SystemShutdownCommand {
    async fn run(&mut self, program: &str, arguments: &[&str]) -> Result<bool> {
        Ok(Command::new(program)
            .args(arguments)
            .status()
            .await
            .context("end job VM")?
            .success())
    }
}

pub(crate) struct VmPower {
    pub(crate) platform: Platform,
    pub(crate) command: Box<dyn ShutdownCommand>,
}

#[async_trait]
impl Power for VmPower {
    async fn shutdown(&mut self) -> Result<()> {
        let (program, arguments) = self.platform.end_command();
        if self.command.run(program, arguments).await? {
            Ok(())
        } else {
            Err(Error::PoweroffFailed)
        }
    }
}

pub(crate) struct SandboxPower;

#[async_trait]
impl Power for SandboxPower {
    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "_tests_/power_tests.rs"]
mod power_tests;
