//! In-memory proxy configuration and acknowledged machine-session replacement.

use std::process::Stdio;
use std::time::Duration;

use async_trait::async_trait;
use ciproxy_protocol::cache::CacheConnection;
use ciproxy_protocol::runtime::{CacheProxyConfig, CacheProxyUpdate};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::time::timeout;

use crate::error::{Error, ErrorOptionExt, ErrorResultExt, Result};

#[cfg_attr(test, unimock::unimock(api = ProxySessionMock))]
#[async_trait]
pub(crate) trait ProxySession: Send {
    async fn replace_connection(&mut self, connection: CacheConnection) -> Result<()>;
}

struct RunningProxy {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
    sequence: u64,
}

pub(crate) async fn start(config: CacheProxyConfig) -> Result<Box<dyn ProxySession>> {
    let mut child = Command::new("/usr/local/bin/ciproxy-cache-sidecar")
        .args([
            "proxy",
            "--listen",
            "0.0.0.0:8230",
            "--config-stdin",
            "--ready-stdout",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .context("start VM cache proxy")?;
    let input = child.stdin.take().ok_or_internal("proxy stdin")?;
    let output = BufReader::new(child.stdout.take().ok_or_internal("proxy stdout")?);
    let mut proxy = RunningProxy {
        child,
        input,
        output,
        sequence: 0,
    };
    let mut bytes = serde_json::to_vec(&config).context("encode in-memory proxy configuration")?;
    bytes.push(b'\n');
    proxy
        .input
        .write_all(&bytes)
        .await
        .context("configure VM proxy")?;
    proxy.acknowledge("READY").await?;
    Ok(Box::new(proxy))
}

impl RunningProxy {
    async fn acknowledge(&mut self, expected: &str) -> Result<()> {
        timeout(Duration::from_secs(10), async {
            loop {
                let mut reply = String::new();
                let bytes = self
                    .output
                    .read_line(&mut reply)
                    .await
                    .context("read proxy acknowledgement")?;
                if reply.trim() == expected {
                    return Ok(());
                }
                if bytes == 0 {
                    return Err(Error::PoweroffFailed);
                }
            }
        })
        .await
        .context("wait for proxy acknowledgement timeout")?
    }
}

#[async_trait]
impl ProxySession for RunningProxy {
    async fn replace_connection(&mut self, connection: CacheConnection) -> Result<()> {
        if self.child.try_wait().context("observe VM proxy")?.is_some() {
            return Err(Error::PoweroffFailed);
        }
        self.sequence = self.sequence.saturating_add(1);
        let update = CacheProxyUpdate {
            sequence: self.sequence,
            connection,
        };
        let mut bytes = serde_json::to_vec(&update).context("encode proxy session update")?;
        bytes.push(b'\n');
        self.input
            .write_all(&bytes)
            .await
            .context("update proxy session")?;
        self.acknowledge(&format!("UPDATED {}", self.sequence))
            .await
    }
}
