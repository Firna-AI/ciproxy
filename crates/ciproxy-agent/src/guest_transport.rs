//! Bounded Firecracker vsock framing behind the guest connection boundary.

use std::future::Future;
use std::pin::Pin;

use async_trait::async_trait;
use ciproxy_protocol::guest::{AgentToGuest, GuestToAgent};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::time::{Duration, timeout};

use crate::error::{Error, ErrorResultExt, Result};
use crate::layout::Layout;
use crate::state::VmRecord;

#[cfg_attr(test, unimock::unimock(api = GuestTransportMock))]
#[async_trait]
pub(crate) trait GuestTransport: Send + Sync {
    async fn connect(&self, vm: &VmRecord) -> Result<Box<dyn GuestSocket>>;
}

/// Cancellation-safe receive future for one guest frame.
pub type GuestRead<'a> = Pin<Box<dyn Future<Output = Result<GuestToAgent>> + Send + 'a>>;

/// One bounded connection to an owned guest.
#[cfg_attr(test, unimock::unimock(api = GuestSocketMock))]
#[async_trait]
pub trait GuestSocket: Send + Sync {
    /// Receive one cancellation-safe guest frame.
    fn receive(&mut self) -> GuestRead<'_>;
    /// Send one agent frame without writing its credentials to disk.
    async fn send(&mut self, message: &AgentToGuest) -> Result<()>;
}

pub(crate) struct UnixGuestTransport {
    pub(crate) layout: Layout,
}

struct Socket {
    reader: BufReader<OwnedReadHalf>,
    writer: OwnedWriteHalf,
    bytes: Vec<u8>,
}

#[async_trait]
impl GuestTransport for UnixGuestTransport {
    async fn connect(&self, vm: &VmRecord) -> Result<Box<dyn GuestSocket>> {
        let stream = UnixStream::connect(self.layout.jail_root(vm).join("vsock.sock"))
            .await
            .context("connect VM vsock")?;
        let (read, mut writer) = stream.into_split();
        let mut reader = BufReader::new(read);
        writer
            .write_all(b"CONNECT 5252\n")
            .await
            .context("connect guest port")?;
        let mut ack = String::new();
        timeout(
            Duration::from_secs(10),
            (&mut reader).take(4096).read_line(&mut ack),
        )
        .await
        .context("wait for Firecracker vsock handshake")?
        .context("read vsock handshake")?;
        if !ack.starts_with("OK ") {
            return Err(Error::GuestMismatch);
        }
        Ok(Box::new(Socket {
            reader,
            writer,
            bytes: Vec::new(),
        }))
    }
}

#[async_trait]
impl GuestSocket for Socket {
    fn receive(&mut self) -> GuestRead<'_> {
        Box::pin(async move {
            let mut bounded =
                (&mut self.reader).take(1_048_576_u64.saturating_sub(self.bytes.len() as u64));
            if bounded
                .read_until(b'\n', &mut self.bytes)
                .await
                .context("read guest event")?
                == 0
            {
                return Err(Error::Disconnected);
            }
            if self.bytes.last() != Some(&b'\n') {
                return Err(Error::GuestMismatch);
            }
            let message = serde_json::from_slice(&self.bytes).context("decode guest event")?;
            self.bytes.clear();
            Ok(message)
        })
    }
    async fn send(&mut self, message: &AgentToGuest) -> Result<()> {
        let mut bytes = serde_json::to_vec(message).context("encode guest message")?;
        bytes.push(b'\n');
        self.writer
            .write_all(&bytes)
            .await
            .context("send guest message")
    }
}
