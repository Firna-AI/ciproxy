//! The Firecracker CONNECT handshake precedes newline-delimited protocol JSON.

use std::sync::Arc;
use std::time::Duration;

use ciproxy_protocol::guest::{AgentToGuest, GuestToAgent};
use ciproxy_protocol::runners::RunnerExit;
use ciproxy_protocol::version::ProtocolVersion;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::mpsc;
use tokio::time::{MissedTickBehavior, interval, sleep, timeout};

use crate::boundaries::{Cache, GuestEvent};
use crate::error::{Error, ErrorResultExt, Result};
use crate::layout::{Layout, unit_name};
use crate::process::{Commands, UnitState};
use crate::state::VmRecord;

pub(crate) struct GuestBridge {
    pub(crate) layout: Layout,
    pub(crate) commands: Arc<dyn Commands>,
    pub(crate) cache: Arc<dyn Cache>,
    pub(crate) events: mpsc::Sender<GuestEvent>,
    pub(crate) vm: VmRecord,
}

impl GuestBridge {
    pub(crate) async fn run(
        self,
        mut config: Option<AgentToGuest>,
        mut receiver: mpsc::Receiver<AgentToGuest>,
    ) {
        loop {
            match self.commands.unit(&unit_name(&self.vm)).await {
                Ok(UnitState::Exited(code)) => {
                    let _ = self
                        .emit(GuestToAgent::RunnerExited {
                            runner_id: self.vm.report.runner_id.clone(),
                            exit: RunnerExit { code: Some(code) },
                        })
                        .await;
                    return;
                }
                Ok(UnitState::Absent) => {
                    let _ = self
                        .emit(GuestToAgent::RunnerExited {
                            runner_id: self.vm.report.runner_id.clone(),
                            exit: RunnerExit { code: None },
                        })
                        .await;
                    return;
                }
                Ok(UnitState::Active) => {}
                Err(error) => tracing::warn!(error = %error, "cannot inspect VM; preserve it"),
            }
            if let Err(error) = self.session(&mut config, &mut receiver).await {
                tracing::debug!(error = %error, "guest channel unavailable; preserve VM and reconnect");
            }
            sleep(Duration::from_secs(1)).await;
        }
    }

    async fn session(
        &self,
        config: &mut Option<AgentToGuest>,
        receiver: &mut mpsc::Receiver<AgentToGuest>,
    ) -> Result<()> {
        let stream = UnixStream::connect(self.layout.jail_root(&self.vm).join("vsock.sock"))
            .await
            .context("connect VM vsock")?;
        let (read, mut write) = stream.into_split();
        let mut read = BufReader::new(read);
        write
            .write_all(b"CONNECT 52\n")
            .await
            .context("connect guest port")?;
        let mut ack = String::new();
        timeout(
            Duration::from_secs(10),
            (&mut read).take(4096).read_line(&mut ack),
        )
        .await
        .context("wait for Firecracker vsock handshake")?
        .context("read vsock handshake")?;
        if !ack.starts_with("OK ") {
            return Err(Error::GuestMismatch);
        }
        let mut hello = Vec::new();
        timeout(
            Duration::from_secs(30),
            (&mut read).take(1_048_576).read_until(b'\n', &mut hello),
        )
        .await
        .context("wait for guest hello")?
        .context("read guest hello")?;
        let message = serde_json::from_slice(&hello).context("decode guest hello")?;
        if message
            != (GuestToAgent::Hello {
                protocol_version: ProtocolVersion::CURRENT,
                image_id: self.vm.report.image_id.clone(),
            })
        {
            return Err(Error::GuestMismatch);
        }
        self.emit(message).await?;
        if let Some(mut configure) = config.clone() {
            if let AgentToGuest::ConfigureRunner { cache, .. } = &mut configure {
                *cache = self.cache.configure(&self.vm).await.unwrap_or_else(|error| {
                    tracing::warn!(error = %error, "machine cache unavailable; use GitHub cache");
                    None
                });
            }
            let mut bytes = serde_json::to_vec(&configure).context("encode registration")?;
            bytes.push(b'\n');
            write
                .write_all(&bytes)
                .await
                .context("send registration over vsock")?;
            *config = None;
        }
        let mut health = interval(Duration::from_secs(10));
        health.set_missed_tick_behavior(MissedTickBehavior::Skip);
        let mut bytes = Vec::new();
        loop {
            let mut bounded = (&mut read).take(1_048_576_u64.saturating_sub(bytes.len() as u64));
            tokio::select! {
                read = bounded.read_until(b'\n', &mut bytes) => {
                    if read.context("read guest event")? == 0 { return Err(Error::Disconnected); }
                    if bytes.last() != Some(&b'\n') { return Err(Error::GuestMismatch); }
                    let message = serde_json::from_slice(&bytes).context("decode guest event")?;
                    self.emit(message).await?;
                    bytes.clear();
                }
                message = receiver.recv() => {
                    let message = message.ok_or(Error::Disconnected)?;
                    let mut bytes = serde_json::to_vec(&message).context("encode guest message")?;
                    bytes.push(b'\n');
                    write.write_all(&bytes).await.context("send guest message")?;
                }
                _ = health.tick() => {
                    if self.commands.unit(&unit_name(&self.vm)).await? != UnitState::Active {
                        return Err(Error::Disconnected);
                    }
                }
            }
        }
    }

    async fn emit(&self, message: GuestToAgent) -> Result<()> {
        self.events
            .send(GuestEvent {
                runner_id: self.vm.report.runner_id.clone(),
                message,
            })
            .await
            .context("deliver guest event")?;
        Ok(())
    }
}
