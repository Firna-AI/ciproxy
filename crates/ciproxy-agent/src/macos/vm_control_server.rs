//! The VM helper's side of the control socket.

use std::io;
use std::os::fd::{AsFd, OwnedFd};
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::net::{UnixListener, UnixStream};
use tokio::time::{Duration, timeout};

use super::fd_passing::{receive_line, send_line};
use super::vm_control::{Answer, ControlError, Request, StopReason, VmPhase};

/// A client must send its request line within this time.
const REQUEST_WAIT: Duration = Duration::from_secs(10);

/// The VM that one helper owns, as the control socket sees it.
#[cfg_attr(test, unimock::unimock(api = VmControlMock))]
#[async_trait]
pub(crate) trait VmControl: Send + Sync {
    fn phase(&self) -> VmPhase;
    async fn connect(&self, port: u32) -> Result<OwnedFd, ControlError>;
    async fn request_stop(&self) -> Result<(), ControlError>;
    async fn save(&self) -> Result<(), ControlError>;
    async fn stopped(&self) -> StopReason;
}

/// Bind the socket with mode `0600`. Its folder is root-only, mode `0700`.
/// A stale socket from an earlier helper is removed. Other files stay.
pub(crate) fn bind(path: &Path) -> io::Result<UnixListener> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_socket() => std::fs::remove_file(path)?,
        Ok(_) => return Err(io::ErrorKind::AlreadyExists.into()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    let listener = UnixListener::bind(path)?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    Ok(listener)
}

/// Answer each connection in its own task until the listener fails.
pub(crate) async fn serve(listener: UnixListener, control: Arc<dyn VmControl>) -> io::Result<()> {
    loop {
        let (stream, _) = listener.accept().await?;
        let control = control.clone();
        tokio::spawn(async move {
            if let Err(error) = answer(&stream, control.as_ref()).await {
                tracing::debug!(%error, "VM control request failed");
            }
        });
    }
}

/// Read one request and send its answer. `CONNECT` passes the stream fd.
pub(crate) async fn answer(stream: &UnixStream, control: &dyn VmControl) -> io::Result<()> {
    let (line, _) = timeout(REQUEST_WAIT, receive_line(stream))
        .await
        .map_err(|_| io::Error::from(io::ErrorKind::TimedOut))??;
    let answer = match Request::parse(&line) {
        None => Answer::Refused(ControlError::Unsupported),
        Some(Request::State) => Answer::Phase(control.phase()),
        Some(Request::Connect(port)) => match control.connect(port).await {
            Ok(fd) => {
                return send_line(stream, Answer::Done.line().as_bytes(), Some(fd.as_fd())).await;
            }
            Err(error) => Answer::Refused(error),
        },
        Some(Request::Stop) => done(control.request_stop().await),
        Some(Request::Save) => done(control.save().await),
        Some(Request::Wait) => Answer::Stopped(control.stopped().await),
    };
    send_line(stream, answer.line().as_bytes(), None).await
}

fn done(result: Result<(), ControlError>) -> Answer {
    result.map_or_else(Answer::Refused, |()| Answer::Done)
}
