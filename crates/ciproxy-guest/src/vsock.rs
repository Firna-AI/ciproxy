//! Reconnectable agent vsock channel with the current guest state replayed.

use async_trait::async_trait;
use ciproxy_protocol::guest::GuestToAgent;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines, ReadHalf, WriteHalf, split};
use tokio_vsock::{VMADDR_CID_ANY, VsockAddr, VsockListener, VsockStream};

use crate::boundary::{Channel, CommandFuture};
use crate::error::{ErrorResultExt, Result};
use crate::replay::Observations;

const MAX_AGENT_LINE_BYTES: usize = 1_048_576;

struct Connection {
    reader: Lines<BufReader<ReadHalf<VsockStream>>>,
    writer: WriteHalf<VsockStream>,
}

pub(crate) struct VsockChannel {
    port: u32,
    listener: Option<VsockListener>,
    connection: Option<Connection>,
    observations: Observations,
}

impl VsockChannel {
    pub(crate) async fn accept(port: u32) -> Result<Self> {
        Ok(Self {
            port,
            listener: Some(bind(port)?),
            connection: None,
            observations: Observations::default(),
        })
    }

    async fn connected(&mut self) -> Result<()> {
        if self.connection.is_some() {
            return Ok(());
        }
        let listener = match &mut self.listener {
            Some(listener) => listener,
            None => self.listener.insert(bind(self.port)?),
        };
        let (stream, _) = match listener.accept().await {
            Ok(accepted) => accepted,
            Err(error) => {
                // A restored macOS VM has a new socket device. Its listener
                // from before the save can fail, so a new one replaces it.
                self.listener = None;
                return Err(error).context("accept agent vsock");
            }
        };
        let (reader, writer) = split(stream);
        self.connection = Some(Connection {
            reader: BufReader::new(reader).lines(),
            writer,
        });
        for observation in self.observations.replay() {
            self.write(observation).await?;
        }
        Ok(())
    }

    async fn write(&mut self, message: GuestToAgent) -> Result<()> {
        let bytes = encode(message)?;
        if let Some(connection) = &mut self.connection
            && let Err(error) = connection.writer.write_all(&bytes).await
        {
            self.connection = None;
            return Err(error).context("write guest observation");
        }
        Ok(())
    }
}

fn encode(mut message: GuestToAgent) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec(&message).context("encode guest observation")?;
    if bytes.len() >= MAX_AGENT_LINE_BYTES
        && let GuestToAgent::RunnerExited {
            usage: Some(usage), ..
        } = &mut message
        && !usage.steps.is_empty()
    {
        usage.steps.clear();
        tracing::warn!("runner steps exceed guest message capacity");
        bytes = serde_json::to_vec(&message).context("encode guest observation")?;
    }
    bytes.push(b'\n');
    Ok(bytes)
}

fn bind(port: u32) -> Result<VsockListener> {
    VsockListener::bind(VsockAddr::new(VMADDR_CID_ANY, port)).context("bind guest vsock")
}

#[cfg(test)]
#[path = "_tests_/vsock_tests.rs"]
mod vsock_tests;

#[async_trait]
impl Channel for VsockChannel {
    fn receive(&mut self) -> CommandFuture<'_> {
        Box::pin(async move {
            loop {
                if let Err(error) = self.connected().await {
                    tracing::warn!(%error, cause = %error.diagnosis(), "agent channel will reconnect");
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    continue;
                }
                if let Some(connection) = &mut self.connection {
                    match connection.reader.next_line().await {
                        Ok(Some(line)) => match serde_json::from_str(&line) {
                            Ok(message) => return Ok(Some(message)),
                            Err(error) => {
                                tracing::warn!(%error, "agent command unavailable; reconnect without releasing the Worker");
                                self.connection = None;
                            }
                        },
                        Ok(None) => self.connection = None,
                        Err(error) => {
                            tracing::warn!(%error, "agent connection failed; wait for a new one");
                            self.connection = None;
                        }
                    }
                }
            }
        })
    }

    async fn send(&mut self, message: GuestToAgent) -> Result<()> {
        if !self.observations.record(&message) {
            return Ok(());
        }
        self.write(message).await
    }
}
