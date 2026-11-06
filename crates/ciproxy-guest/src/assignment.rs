//! Own the trusted assignment read independently of supervisor cancellation.

use std::future::Future;
use std::pin::Pin;

use async_trait::async_trait;
use ciproxy_protocol::guest::HANDOFF_PROTOCOL_VERSION;
use ciproxy_protocol::identifiers::JobId;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::error::{Error, ErrorResultExt, Result};
use crate::guest_setup::PreloadedDocker;
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
            loop {
                let assignment = source.receive().await;
                let failed = assignment.is_err();
                if sender.send(assignment).await.is_err() {
                    return;
                }
                if failed {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
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
    pub(crate) preloaded: PreloadedDocker,
}

impl AssignmentSource for SocketAssignments {
    fn receive(&mut self) -> AssignmentFuture<'_> {
        Box::pin(async move {
            let (stream, _) = self
                .listener
                .accept()
                .await
                .context("accept trusted assignment")?;
            let (handoff, stream) = read(stream, self.preloaded.clone()).await?;
            let job_id = handoff.job_id.clone();
            Ok(Assignment {
                handoff,
                reply: Box::new(SocketReply { stream, job_id }),
            })
        })
    }
}

struct SocketReply {
    stream: UnixStream,
    job_id: JobId,
}

#[derive(Serialize)]
struct Ready<'a> {
    handoff_protocol_version: u16,
    job_id: &'a JobId,
    decision: ReadyDecision,
    #[serde(skip_serializing_if = "Option::is_none")]
    proxy_url: Option<&'a str>,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum ReadyDecision {
    GithubOnly,
    CacheReady,
}

#[derive(Serialize)]
struct Preparing<'a> {
    handoff_protocol_version: u16,
    job_id: &'a JobId,
    preparing: bool,
}

#[derive(Deserialize)]
struct Acknowledged {
    handoff_protocol_version: u16,
    job_id: JobId,
    acknowledged: bool,
}

#[async_trait]
impl WorkerReply for SocketReply {
    async fn begin(&mut self) -> Result<bool> {
        let mut offer = serde_json::to_vec(&Preparing {
            handoff_protocol_version: HANDOFF_PROTOCOL_VERSION,
            job_id: &self.job_id,
            preparing: true,
        })
        .context("encode host-authorized preparation")?;
        offer.push(b'\n');
        self.stream
            .write_all(&offer)
            .await
            .context("offer local job preparation")?;
        let mut line = String::new();
        BufReader::new(&mut self.stream)
            .read_line(&mut line)
            .await
            .context("read preparation acknowledgement")?;
        let acknowledged: Acknowledged =
            serde_json::from_str(&line).context("decode preparation acknowledgement")?;
        Ok(
            acknowledged.handoff_protocol_version == HANDOFF_PROTOCOL_VERSION
                && acknowledged.job_id == self.job_id
                && acknowledged.acknowledged,
        )
    }
    async fn ready(&mut self, proxy_url: Option<&str>) -> Result<()> {
        let mut bytes = serde_json::to_vec(&Ready {
            handoff_protocol_version: HANDOFF_PROTOCOL_VERSION,
            job_id: &self.job_id,
            decision: if proxy_url.is_some() {
                ReadyDecision::CacheReady
            } else {
                ReadyDecision::GithubOnly
            },
            proxy_url,
        })
        .context("encode job readiness")?;
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
