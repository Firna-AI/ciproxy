//! Root systemd service installation. VM units have independent lifetimes.

use std::fs;
use std::path::Path;
use std::process::Command;

use crate::boundaries::Service;
use crate::error::{Error, ErrorResultExt, Result};
use crate::persistence::private_directory;

pub(crate) struct SystemService;

const UNIT_PATH: &str = "/etc/systemd/system/ciproxy-agent.service";
const BINARY: &str = "/usr/local/bin/ciproxy-agent";
const UNIT: &str = "[Unit]\nDescription=ciproxy machine agent\nAfter=network-online.target\nWants=network-online.target\n\n[Service]\nType=simple\nUser=root\nGroup=root\nEnvironmentFile=-/etc/ciproxy-agent/environment\nExecStart=/usr/local/bin/ciproxy-agent run\nRestart=on-failure\nRestartSec=5\nKillMode=process\nStateDirectory=ciproxy\nStateDirectoryMode=0700\nRuntimeDirectory=ciproxy-agent\nRuntimeDirectoryMode=0700\n\n[Install]\nWantedBy=multi-user.target\n";

impl Service for SystemService {
    fn install(&self) -> Result<()> {
        private_directory(Path::new("/var/lib/ciproxy"))?;
        let current = std::env::current_exe().context("locate agent binary")?;
        if current != Path::new(BINARY) {
            let pending = "/usr/local/bin/ciproxy-agent.pending";
            fs::copy(&current, pending).context("stage agent binary")?;
            fs::rename(pending, BINARY).context("install agent binary")?;
        }
        fs::write(UNIT_PATH, UNIT).context("write agent service")?;
        systemctl(&["daemon-reload"])?;
        systemctl(&["enable", "--now", "ciproxy-agent.service"])?;
        systemctl(&["try-restart", "ciproxy-agent.service"])
    }

    fn stop(&self) -> Result<()> {
        let output = Command::new("systemctl")
            .args([
                "show",
                "--property=LoadState",
                "--value",
                "ciproxy-agent.service",
            ])
            .output()
            .context("inspect agent service")?;
        if String::from_utf8_lossy(&output.stdout).trim() == "not-found" {
            return Ok(());
        }
        systemctl(&["stop", "ciproxy-agent.service"])
    }

    fn uninstall(&self) -> Result<()> {
        self.stop()?;
        systemctl(&["disable", "ciproxy-agent.service"])?;
        for path in [UNIT_PATH, BINARY] {
            match fs::remove_file(path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error).context("remove agent installation"),
            }
        }
        systemctl(&["daemon-reload"])
    }
}

fn systemctl(args: &[&str]) -> Result<()> {
    let status = Command::new("systemctl")
        .args(args)
        .status()
        .context("control agent service")?;
    if !status.success() {
        return Err(Error::HostCommand {
            program: "systemctl".to_owned(),
        });
    }
    Ok(())
}
