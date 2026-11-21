//! Mockable host observations and subprocess outcomes for installation only.

use std::fs::{self, OpenOptions};

use async_trait::async_trait;
use tokio::process::Command;

use crate::error::{Error, ErrorResultExt, Result};

#[derive(Clone, Debug)]
/// Exit code and stdout from one host preparation tool.
pub struct HostOutput {
    pub(crate) code: i32,
    pub(crate) stdout: Vec<u8>,
}

#[cfg_attr(test, unimock::unimock(api = HostSystemMock))]
#[async_trait]
pub(crate) trait HostSystem: Send + Sync {
    fn read(&self, path: &str) -> Result<Vec<u8>>;
    fn kvm(&self) -> bool;
    async fn execute(&self, program: &str, args: &[String]) -> Result<HostOutput>;
}

pub(crate) struct LinuxHostSystem;

#[async_trait]
impl HostSystem for LinuxHostSystem {
    fn kvm(&self) -> bool {
        OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/kvm")
            .is_ok()
    }

    fn read(&self, path: &str) -> Result<Vec<u8>> {
        fs::read(path).context("read host preparation input")
    }

    async fn execute(&self, program: &str, args: &[String]) -> Result<HostOutput> {
        let output = Command::new(program)
            .args(args)
            .env("DEBIAN_FRONTEND", "noninteractive")
            .env("LC_ALL", "C")
            .kill_on_drop(true)
            .output()
            .await
            .context("run host preparation tool")?;
        Ok(HostOutput {
            code: output.status.code().unwrap_or(-1),
            stdout: output.stdout,
        })
    }
}

pub(crate) fn repair(component: &'static str, repair: &'static str) -> Error {
    Error::Preparation { component, repair }
}
