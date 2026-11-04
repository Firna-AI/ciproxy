//! Newline-delimited protocol JSON on one agent-to-guest vsock stream.

use async_trait::async_trait;
use ciproxy_protocol::guest::GuestToAgent;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines, ReadHalf, WriteHalf, split};
use tokio_vsock::{VMADDR_CID_ANY, VsockAddr, VsockListener, VsockStream};

use crate::boundary::{Channel, CommandFuture};
use crate::error::{ErrorResultExt, Result};

pub(crate) struct VsockChannel {
    reader: Lines<BufReader<ReadHalf<VsockStream>>>,
    writer: WriteHalf<VsockStream>,
}

impl VsockChannel {
    pub(crate) async fn accept(port: u32) -> Result<Self> {
        let listener = VsockListener::bind(VsockAddr::new(VMADDR_CID_ANY, port))
            .context("bind guest vsock")?;
        let (stream, _) = listener.accept().await.context("accept agent vsock")?;
        let (reader, writer) = split(stream);
        Ok(Self {
            reader: BufReader::new(reader).lines(),
            writer,
        })
    }
}

#[async_trait]
impl Channel for VsockChannel {
    fn receive(&mut self) -> CommandFuture<'_> {
        Box::pin(async move {
            match self.reader.next_line().await.context("read vsock")? {
                Some(line) => Ok(Some(
                    serde_json::from_str(&line).context("decode agent command")?,
                )),
                None => Ok(None),
            }
        })
    }

    async fn send(&mut self, message: GuestToAgent) -> Result<()> {
        let mut bytes = serde_json::to_vec(&message).context("encode observation")?;
        bytes.push(b'\n');
        self.writer.write_all(&bytes).await.context("write vsock")?;
        self.writer.flush().await.context("flush vsock")
    }
}
