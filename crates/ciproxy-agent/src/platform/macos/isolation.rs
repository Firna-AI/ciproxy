//! pf isolation of the guests: load, check, and remove the anchor.

use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::process::Stdio;

use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::sync::Mutex;

use crate::error::{Error, ErrorResultExt, Result};
use crate::macos::pf::{
    DEFAULT_BRIDGE, DEFAULT_GATEWAY, DEFAULT_MASK, VmNetwork, enable_token, enabled, pfctl, rules,
    same_rules,
};
use crate::persistence::atomic_private_write;

const VMNET_SETTINGS: &str = "/Library/Preferences/SystemConfiguration/com.apple.vmnet.plist";

#[async_trait]
pub(crate) trait Isolation: Send + Sync {
    /// Turn pf on and load the rules when they are missing or changed.
    /// A start fails when this fails, so no guest runs without the rules.
    async fn ensure(&self) -> Result<()>;
    /// Flush the anchor and give the pf reference back.
    async fn remove(&self) -> Result<()>;
}

pub(crate) struct PfIsolation {
    pub(crate) machine_cache: bool,
    /// The pf reference token, kept across agent restarts.
    pub(crate) token_file: PathBuf,
    /// pf's own form of the rules just after the last load.
    snapshot: Mutex<Option<String>>,
}

impl PfIsolation {
    pub(crate) fn new(machine_cache: bool, token_file: PathBuf) -> Self {
        Self {
            machine_cache,
            token_file,
            snapshot: Mutex::new(None),
        }
    }
}

#[async_trait]
impl Isolation for PfIsolation {
    async fn ensure(&self) -> Result<()> {
        let mut snapshot = self.snapshot.lock().await;
        if !enabled(&pfctl_text(&pfctl::info(), None).await?) {
            let output = pfctl_text(&pfctl::enable(), None).await?;
            let token = enable_token(&output).ok_or_else(|| pf_error("read the pf token"))?;
            atomic_private_write(&self.token_file, token.to_string().as_bytes())?;
        }
        let shown = pfctl_text(&pfctl::show(), None).await?;
        if snapshot
            .as_deref()
            .is_some_and(|expected| same_rules(expected, &shown))
        {
            return Ok(());
        }
        let network = vmnet_network().await;
        let text = rules(&network, self.machine_cache);
        pfctl_text(&pfctl::load(), Some(text.as_bytes())).await?;
        let shown = pfctl_text(&pfctl::show(), None).await?;
        if shown.trim().is_empty() {
            return Err(pf_error("load the guest isolation rules"));
        }
        tracing::info!(subnet = %network.subnet(), bridge = %network.bridge, "pf rules loaded");
        *snapshot = Some(shown);
        Ok(())
    }

    async fn remove(&self) -> Result<()> {
        pfctl_text(&pfctl::flush(), None).await?;
        let token = match std::fs::read_to_string(&self.token_file) {
            Ok(token) => token.trim().parse::<u64>().ok(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error).context("read the pf token"),
        };
        if let Some(token) = token
            && let Err(error) = pfctl_text(&pfctl::release(token), None).await
        {
            tracing::warn!(%error, "the pf token was gone, for example after a reboot");
        }
        match std::fs::remove_file(&self.token_file) {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
                Err(error).context("remove the pf token")
            }
            _ => Ok(()),
        }
    }
}

/// The vmnet shared network from its settings, else the vmnet defaults.
async fn vmnet_network() -> VmNetwork {
    let address = plist_value("Shared_Net_Address").await;
    let mask = plist_value("Shared_Net_Mask").await;
    let gateway = address.unwrap_or(DEFAULT_GATEWAY);
    VmNetwork::new(gateway, mask.unwrap_or(DEFAULT_MASK), DEFAULT_BRIDGE).unwrap_or_else(|| {
        tracing::warn!("the vmnet settings are invalid, so the rules use the defaults");
        VmNetwork::new(DEFAULT_GATEWAY, DEFAULT_MASK, DEFAULT_BRIDGE)
            .expect("the vmnet defaults are a valid network")
    })
}

async fn plist_value(key: &str) -> Option<Ipv4Addr> {
    let output = Command::new("plutil")
        .args(["-extract", key, "raw", "-o", "-", VMNET_SETTINGS])
        .output()
        .await
        .ok()?;
    output.status.success().then_some(())?;
    String::from_utf8_lossy(&output.stdout).trim().parse().ok()
}

/// Run `pfctl` and return its standard output and standard error. pfctl
/// writes the token and the status lines to standard error.
async fn pfctl_text(args: &[String], input: Option<&[u8]>) -> Result<String> {
    let mut child = Command::new("pfctl")
        .args(args)
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("run pfctl")?;
    if let (Some(input), Some(mut stdin)) = (input, child.stdin.take()) {
        stdin.write_all(input).await.context("send pf rules")?;
    }
    let output = child.wait_with_output().await.context("wait for pfctl")?;
    if !output.status.success() {
        tracing::warn!(
            stderr = %String::from_utf8_lossy(&output.stderr).trim(),
            "pfctl failed"
        );
        return Err(pf_error("run pfctl"));
    }
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    Ok(text)
}

fn pf_error(step: &'static str) -> Error {
    tracing::warn!(step, "a pf step failed");
    Error::HostCommand {
        program: "pfctl".to_owned(),
    }
}
