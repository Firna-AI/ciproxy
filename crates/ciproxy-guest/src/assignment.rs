//! Own the trusted assignment read independently of supervisor cancellation.

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use async_trait::async_trait;
use serde::Serialize;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::timeout;

use crate::error::{Error, ErrorResultExt, Result};
use crate::handoff::{Handoff, read};

/// One original assignment and its private readiness response channel.
pub struct Assignment {
    pub(crate) handoff: Handoff,
    pub(crate) reply: Box<dyn WorkerReply>,
}

/// Cancellable receipt of an assignment already owned by the inbox task.
pub type AssignmentFuture<'a> = Pin<Box<dyn Future<Output = Result<Assignment>> + Send + 'a>>;

#[cfg_attr(test, unimock::unimock(api = AssignmentSourceMock))]
pub(crate) trait AssignmentSource: Send {
    fn receive(&mut self) -> AssignmentFuture<'_>;
}

#[cfg_attr(test, unimock::unimock(api = WorkerReplyMock))]
#[async_trait]
pub(crate) trait WorkerReply: Send {
    async fn begin(&mut self) -> Result<bool>;
    async fn ready(&mut self, proxy_url: Option<&str>) -> Result<()>;
}

pub(crate) trait Assignments: Send {
    fn receive(&mut self) -> AssignmentFuture<'_>;
}

pub(crate) struct Inbox {
    receiver: mpsc::Receiver<Result<Assignment>>,
    reader: JoinHandle<()>,
}

impl Inbox {
    pub(crate) fn new(mut source: Box<dyn AssignmentSource>) -> Self {
        let (sender, receiver) = mpsc::channel(1);
        let reader = tokio::spawn(async move {
            let assignment = source.receive().await;
            let _ = sender.send(assignment).await;
        });
        Self { receiver, reader }
    }
}

impl Assignments for Inbox {
    fn receive(&mut self) -> AssignmentFuture<'_> {
        Box::pin(async move {
            self.receiver
                .recv()
                .await
                .ok_or_else(|| Error::internal_message("assignment source closed"))?
        })
    }
}

impl Drop for Inbox {
    fn drop(&mut self) {
        self.reader.abort();
    }
}

pub(crate) struct SocketAssignments {
    pub(crate) listener: UnixListener,
}

impl AssignmentSource for SocketAssignments {
    fn receive(&mut self) -> AssignmentFuture<'_> {
        Box::pin(async move {
            let (stream, _) = self
                .listener
                .accept()
                .await
                .context("accept trusted assignment")?;
            let (handoff, stream) = read(stream).await?;
            Ok(Assignment {
                handoff,
                reply: Box::new(SocketReply { stream }),
            })
        })
    }
}

struct SocketReply {
    stream: UnixStream,
}

#[derive(Serialize)]
struct Ready<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    proxy_url: Option<&'a str>,
}

#[async_trait]
impl WorkerReply for SocketReply {
    async fn begin(&mut self) -> Result<bool> {
        self.stream
            .write_all(b"{\"preparing\":true}\n")
            .await
            .context("offer local job preparation")?;
        let mut acknowledged = String::new();
        timeout(
            Duration::from_secs(10),
            BufReader::new(&mut self.stream).read_line(&mut acknowledged),
        )
        .await
        .context("wait for runner preparation acknowledgement")?
        .context("read runner preparation acknowledgement")?;
        Ok(acknowledged.trim() == "{\"acknowledged\":true}")
    }
    async fn ready(&mut self, proxy_url: Option<&str>) -> Result<()> {
        let mut bytes = serde_json::to_vec(&Ready { proxy_url }).context("encode job readiness")?;
        bytes.push(b'\n');
        self.stream
            .write_all(&bytes)
            .await
            .context("acknowledge job readiness")
    }
}

#[cfg(test)]
#[path = "_tests_/assignment_tests.rs"]
mod assignment_tests;
