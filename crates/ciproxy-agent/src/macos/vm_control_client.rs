//! The agent's side of a VM helper's control socket.

use std::io;
use std::os::fd::OwnedFd;
use std::path::PathBuf;

use tokio::net::UnixStream;
use tokio::time::{Duration, timeout};

use super::fd_passing::{receive_line, send_line};
use super::vm_control::{Answer, ControlError, Request, StopReason, VmPhase};
use crate::error::Error;

/// A vsock connect answers at once, a refused one too.
const CONNECT_WAIT: Duration = Duration::from_secs(30);
/// State and stop requests answer at once.
const SHORT_WAIT: Duration = Duration::from_secs(10);
/// The spike saved 4 GB of memory in 1.8 s. This leaves room for slow disks.
const SAVE_WAIT: Duration = Duration::from_secs(300);

#[derive(Debug, thiserror::Error)]
pub(crate) enum ControlFailure {
    /// The socket is absent or refuses connections, so the helper ended.
    #[error("the VM helper is not running")]
    HelperGone,
    #[error("the VM helper refused the request: {}", .0.word())]
    Refused(ControlError),
    #[error("the VM helper sent an invalid answer")]
    Protocol,
    #[error("the VM helper did not answer in time")]
    TimedOut,
    #[error("VM helper I/O failed: {0}")]
    Io(#[from] io::Error),
}

impl From<ControlFailure> for Error {
    fn from(failure: ControlFailure) -> Self {
        match failure {
            ControlFailure::HelperGone
            | ControlFailure::Refused(ControlError::GuestNotListening | ControlError::Stopped) => {
                Error::Disconnected
            }
            _ => Error::HostCommand {
                program: "ciproxy-agent vm-run".to_owned(),
            },
        }
    }
}

/// One VM helper, reached through its control socket path.
#[derive(Clone, Debug)]
pub(crate) struct VmControlClient {
    pub(crate) socket: PathBuf,
}

impl VmControlClient {
    pub(crate) async fn phase(&self) -> Result<VmPhase, ControlFailure> {
        match self.request(Request::State, Some(SHORT_WAIT)).await? {
            (Answer::Phase(phase), _) => Ok(phase),
            (answer, _) => Err(refused(answer)),
        }
    }

    /// Take the stream of a guest vsock port. The registration then flows
    /// on this stream only, never through the helper.
    pub(crate) async fn connect(&self, port: u32) -> Result<UnixStream, ControlFailure> {
        match self
            .request(Request::Connect(port), Some(CONNECT_WAIT))
            .await?
        {
            (Answer::Done, Some(fd)) => {
                let stream = std::os::unix::net::UnixStream::from(fd);
                stream.set_nonblocking(true)?;
                Ok(UnixStream::from_std(stream)?)
            }
            (answer, _) => Err(refused(answer)),
        }
    }

    pub(crate) async fn request_stop(&self) -> Result<(), ControlFailure> {
        self.done(Request::Stop, SHORT_WAIT).await
    }

    /// Pause the VM and save its state into the VM folder.
    pub(crate) async fn save(&self) -> Result<(), ControlFailure> {
        self.done(Request::Save, SAVE_WAIT).await
    }

    /// Wait with no time limit until the VM stops.
    pub(crate) async fn wait(&self) -> Result<StopReason, ControlFailure> {
        match self.request(Request::Wait, None).await? {
            (Answer::Stopped(reason), _) => Ok(reason),
            (answer, _) => Err(refused(answer)),
        }
    }

    async fn done(&self, request: Request, wait: Duration) -> Result<(), ControlFailure> {
        match self.request(request, Some(wait)).await? {
            (Answer::Done, None) => Ok(()),
            (answer, _) => Err(refused(answer)),
        }
    }

    async fn request(
        &self,
        request: Request,
        wait: Option<Duration>,
    ) -> Result<(Answer, Option<OwnedFd>), ControlFailure> {
        let exchange = async {
            let stream = match UnixStream::connect(&self.socket).await {
                Ok(stream) => stream,
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::NotFound | io::ErrorKind::ConnectionRefused
                    ) =>
                {
                    return Err(ControlFailure::HelperGone);
                }
                Err(error) => return Err(error.into()),
            };
            exchange(&stream, request).await
        };
        match wait {
            Some(wait) => timeout(wait, exchange)
                .await
                .map_err(|_| ControlFailure::TimedOut)?,
            None => exchange.await,
        }
    }
}

/// Send one request on a connected stream and read its answer.
pub(crate) async fn exchange(
    stream: &UnixStream,
    request: Request,
) -> Result<(Answer, Option<OwnedFd>), ControlFailure> {
    send_line(stream, request.line().as_bytes(), None).await?;
    let (line, fd) = receive_line(stream).await?;
    let answer = Answer::parse(request, &line).ok_or(ControlFailure::Protocol)?;
    Ok((answer, fd))
}

fn refused(answer: Answer) -> ControlFailure {
    match answer {
        Answer::Refused(error) => ControlFailure::Refused(error),
        _ => ControlFailure::Protocol,
    }
}
