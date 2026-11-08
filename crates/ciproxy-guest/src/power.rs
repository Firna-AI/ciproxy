//! End an own-machine Firecracker VM or leave a Boat namespace to its wrapper.

use async_trait::async_trait;
use tokio::process::Command;

use crate::boundary::Power;
use crate::error::{Error, ErrorResultExt, Result};

#[cfg_attr(test, unimock::unimock(api = ShutdownCommandMock))]
#[async_trait]
pub(crate) trait ShutdownCommand: Send {
    async fn run(&mut self, program: &str) -> Result<bool>;
}

pub(crate) struct LinuxShutdownCommand;

#[async_trait]
impl ShutdownCommand for LinuxShutdownCommand {
    async fn run(&mut self, program: &str) -> Result<bool> {
        Ok(Command::new(program)
            .status()
            .await
            .context("end job VM")?
            .success())
    }
}

pub(crate) struct LinuxPower {
    pub(crate) command: Box<dyn ShutdownCommand>,
}

#[async_trait]
impl Power for LinuxPower {
    async fn shutdown(&mut self) -> Result<()> {
        if self.command.run("/sbin/reboot").await? {
            Ok(())
        } else {
            Err(Error::PoweroffFailed)
        }
    }
}

pub(crate) struct NamespacePower;

#[async_trait]
impl Power for NamespacePower {
    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "_tests_/power_tests.rs"]
mod power_tests;
