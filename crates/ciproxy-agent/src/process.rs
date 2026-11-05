//! Host subprocess adapter. Inputs use argv or stdin without a shell.

use std::process::Stdio;

use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::error::{Error, ErrorResultExt, Result};

#[cfg_attr(test, unimock::unimock(api = CommandsMock))]
#[async_trait]
pub(crate) trait Commands: Send + Sync {
    async fn run(&self, program: &str, args: &[String], input: &[u8]) -> Result<Vec<u8>>;
    async fn unit(&self, name: &str) -> Result<UnitState>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Unit status used to preserve live VMs and reclaim exited VMs.
pub enum UnitState {
    /// The unit process is live or starting.
    Active,
    /// The unit process exited with this code.
    Exited(i32),
    /// No loaded unit has this locally generated name.
    Absent,
}

pub(crate) struct LinuxCommands;

#[async_trait]
impl Commands for LinuxCommands {
    async fn unit(&self, name: &str) -> Result<UnitState> {
        let output = Command::new("systemctl")
            .args([
                "show",
                "--property=LoadState,ActiveState,ExecMainStatus",
                name,
            ])
            .output()
            .await
            .context("inspect VM unit")?;
        let text = String::from_utf8(output.stdout).context("read VM unit state")?;
        let value = |key| {
            text.lines().find_map(|line| {
                let (name, value) = line.split_once('=')?;
                (name == key).then_some(value)
            })
        };
        if value("LoadState") == Some("not-found") {
            return Ok(UnitState::Absent);
        }
        if !output.status.success() {
            return Err(Error::HostCommand {
                program: "systemctl".to_owned(),
            });
        }
        if matches!(
            value("ActiveState"),
            Some("active" | "activating" | "reloading")
        ) {
            return Ok(UnitState::Active);
        }
        let code = value("ExecMainStatus")
            .unwrap_or("1")
            .parse()
            .context("parse VM exit code")?;
        Ok(UnitState::Exited(code))
    }

    async fn run(&self, program: &str, args: &[String], input: &[u8]) -> Result<Vec<u8>> {
        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .context("start host tool")?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(input)
                .await
                .context("write host tool input")?;
        }
        let output = child
            .wait_with_output()
            .await
            .context("wait for host tool")?;
        if !output.status.success() {
            return Err(Error::HostCommand {
                program: program.to_owned(),
            });
        }
        Ok(output.stdout)
    }
}

pub(crate) fn arguments(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}
