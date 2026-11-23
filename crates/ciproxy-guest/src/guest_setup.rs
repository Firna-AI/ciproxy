//! Measure preloaded Docker facts before the runner can accept its one job.
//! A macOS image has no Docker, so its steps reach the proxy on loopback.

use async_trait::async_trait;
use tokio::process::Command;

use crate::platform::Platform;

#[derive(Clone, Default)]
pub(crate) struct PreloadedDocker {
    pub(crate) proxy_url: String,
    pub(crate) data_bytes: u64,
}

#[async_trait]
pub(crate) trait GuestSetup: Send {
    async fn inspect(&self) -> PreloadedDocker;
}

pub(crate) struct LinuxGuestSetup;

#[async_trait]
impl GuestSetup for LinuxGuestSetup {
    async fn inspect(&self) -> PreloadedDocker {
        let gateway = Command::new("/usr/local/libexec/ciproxy-container-proxy-url")
            .output()
            .await;
        let proxy_url = match gateway {
            Ok(output) if output.status.success() => String::from_utf8(output.stdout)
                .unwrap_or_default()
                .trim()
                .to_owned(),
            _ => String::new(),
        };
        let data = Command::new("du")
            .args(["--summarize", "--block-size=1", "/var/lib/docker"])
            .output()
            .await;
        let data_bytes = match data {
            Ok(output) if output.status.success() => String::from_utf8(output.stdout)
                .unwrap_or_default()
                .split_whitespace()
                .next()
                .and_then(|value| value.parse().ok())
                .unwrap_or(0),
            _ => 0,
        };
        PreloadedDocker {
            proxy_url,
            data_bytes,
        }
    }
}

/// The macOS image: no Docker, and steps run in the VM itself.
pub(crate) struct MacGuestSetup;

#[async_trait]
impl GuestSetup for MacGuestSetup {
    async fn inspect(&self) -> PreloadedDocker {
        PreloadedDocker {
            proxy_url: format!("http://{}", Platform::MacOs.proxy_listen()),
            data_bytes: 0,
        }
    }
}
