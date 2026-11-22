//! One answer line with an optional `SCM_RIGHTS` fd over a Unix stream.

use std::io::{self, IoSlice, IoSliceMut};
use std::mem::MaybeUninit;
use std::os::fd::{BorrowedFd, OwnedFd};

use rustix::net::{
    RecvAncillaryBuffer, RecvAncillaryMessage, RecvFlags, SendAncillaryBuffer,
    SendAncillaryMessage, SendFlags, recvmsg, sendmsg,
};
use tokio::io::Interest;
use tokio::net::UnixStream;

use super::vm_control::LINE_LIMIT;

/// Send `line` in one message. The fd, when present, rides on its first byte.
pub(crate) async fn send_line(
    stream: &UnixStream,
    line: &[u8],
    fd: Option<BorrowedFd<'_>>,
) -> io::Result<()> {
    loop {
        stream.writable().await?;
        let sent = stream.try_io(Interest::WRITABLE, || {
            let mut space = [MaybeUninit::uninit(); rustix::cmsg_space!(ScmRights(1))];
            let mut control = SendAncillaryBuffer::new(&mut space);
            let fds: Vec<BorrowedFd<'_>> = fd.into_iter().collect();
            if !fds.is_empty() && !control.push(SendAncillaryMessage::ScmRights(&fds)) {
                return Err(io::Error::other("fd control space is too small"));
            }
            Ok(sendmsg(
                stream,
                &[IoSlice::new(line)],
                &mut control,
                SendFlags::empty(),
            )?)
        });
        match sent {
            Ok(bytes) if bytes == line.len() => return Ok(()),
            Ok(_) => return Err(io::Error::other("short control write")),
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
            Err(error) => return Err(error),
        }
    }
}

/// Receive one line and at most one fd. Reject a second fd or a long line.
pub(crate) async fn receive_line(stream: &UnixStream) -> io::Result<(String, Option<OwnedFd>)> {
    let mut line = Vec::new();
    let mut passed: Option<OwnedFd> = None;
    while line.last() != Some(&b'\n') {
        stream.readable().await?;
        let mut buffer = [0_u8; LINE_LIMIT];
        let received = stream.try_io(Interest::READABLE, || {
            let mut space = [MaybeUninit::uninit(); rustix::cmsg_space!(ScmRights(1))];
            let mut control = RecvAncillaryBuffer::new(&mut space);
            let message = recvmsg(
                stream,
                &mut [IoSliceMut::new(&mut buffer)],
                &mut control,
                cloexec(),
            )?;
            let mut fds = Vec::new();
            for message in control.drain() {
                if let RecvAncillaryMessage::ScmRights(rights) = message {
                    fds.extend(rights);
                }
            }
            Ok((message.bytes, fds))
        });
        let (bytes, fds) = match received {
            Ok(received) => received,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => continue,
            Err(error) => return Err(error),
        };
        for fd in fds {
            rustix::io::fcntl_setfd(&fd, rustix::io::FdFlags::CLOEXEC)?;
            if passed.replace(fd).is_some() {
                return Err(io::Error::other("more than one passed fd"));
            }
        }
        if bytes == 0 {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }
        line.extend_from_slice(&buffer[..bytes]);
        if line.len() > LINE_LIMIT || line[..line.len() - 1].contains(&b'\n') {
            return Err(io::Error::other("invalid control line"));
        }
    }
    let line = String::from_utf8(line).map_err(|_| io::Error::other("invalid control text"))?;
    Ok((line, passed))
}

/// Linux closes the fd on exec at once. macOS has no flag for it, so
/// `receive_line` also sets the flag after the receive.
fn cloexec() -> RecvFlags {
    #[cfg(target_os = "linux")]
    return RecvFlags::CMSG_CLOEXEC;
    #[cfg(not(target_os = "linux"))]
    return RecvFlags::empty();
}
