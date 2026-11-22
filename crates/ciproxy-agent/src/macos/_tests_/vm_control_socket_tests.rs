//! Socket pairs stand in for the control socket and the guest vsock stream.

use std::os::fd::OwnedFd;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use unimock::{MockFn, Unimock, matching};

use super::fd_passing::{receive_line, send_line};
use super::vm_control::{Answer, ControlError, Request, StopReason, VmPhase};
use super::vm_control_client::{ControlFailure, exchange};
use super::vm_control_server::{VmControlMock, answer};
use crate::error::Error;

async fn ask(mock: Unimock, request: Request) -> Result<(Answer, Option<OwnedFd>), ControlFailure> {
    let (helper, agent) = UnixStream::pair().expect("control pair");
    let served = tokio::spawn(async move { answer(&helper, &mock).await });
    let result = exchange(&agent, request).await;
    served.await.expect("helper task").expect("helper answer");
    result
}

#[tokio::test]
async fn connect_passes_a_working_guest_stream() {
    let (guest, host) = std::os::unix::net::UnixStream::pair().expect("guest pair");
    let host = OwnedFd::from(host);
    let mock = Unimock::new(
        VmControlMock::connect
            .next_call(matching!(5252))
            .answers_arc(std::sync::Arc::new(move |_, _| {
                Ok(host.try_clone().expect("clone host end"))
            })),
    );

    let (result, fd) = ask(mock, Request::Connect(5252)).await.expect("connect");

    assert_eq!(result, Answer::Done);
    let stream = std::os::unix::net::UnixStream::from(fd.expect("passed fd"));
    stream.set_nonblocking(true).expect("nonblocking");
    let mut stream = UnixStream::from_std(stream).expect("tokio stream");
    guest.set_nonblocking(true).expect("nonblocking");
    let mut guest = UnixStream::from_std(guest).expect("tokio guest");
    stream.write_all(b"hello\n").await.expect("write");
    let mut bytes = [0_u8; 6];
    guest.read_exact(&mut bytes).await.expect("read");
    assert_eq!(&bytes, b"hello\n");
}

#[tokio::test]
async fn a_refused_connect_is_a_retryable_disconnect() {
    let mock = Unimock::new(
        VmControlMock::connect
            .next_call(matching!(5252))
            .returns(Err(ControlError::GuestNotListening)),
    );

    let (result, fd) = ask(mock, Request::Connect(5252)).await.expect("answer");

    assert_eq!(result, Answer::Refused(ControlError::GuestNotListening));
    assert!(fd.is_none());
    let error = Error::from(ControlFailure::Refused(ControlError::GuestNotListening));
    assert!(matches!(error, Error::Disconnected));
    assert!(matches!(
        Error::from(ControlFailure::HelperGone),
        Error::Disconnected
    ));
    assert!(matches!(
        Error::from(ControlFailure::Refused(ControlError::Failed)),
        Error::HostCommand { .. }
    ));
}

#[tokio::test]
async fn state_stop_save_and_wait_reach_the_vm() {
    let mock = Unimock::new((
        VmControlMock::phase
            .next_call(matching!())
            .returns(VmPhase::Paused),
        VmControlMock::request_stop
            .next_call(matching!())
            .returns(Ok(())),
        VmControlMock::save
            .next_call(matching!())
            .returns(Err(ControlError::Stopped)),
        VmControlMock::stopped
            .next_call(matching!())
            .returns(StopReason::HostStop),
    ));

    let mut answers = Vec::new();
    for request in [Request::State, Request::Stop, Request::Save, Request::Wait] {
        answers.push(ask(mock.clone(), request).await.expect("answer").0);
    }

    assert_eq!(
        answers,
        [
            Answer::Phase(VmPhase::Paused),
            Answer::Done,
            Answer::Refused(ControlError::Stopped),
            Answer::Stopped(StopReason::HostStop),
        ]
    );
}

#[tokio::test]
async fn an_unknown_request_gets_unsupported() {
    let (helper, agent) = UnixStream::pair().expect("control pair");
    let served = tokio::spawn(async move { answer(&helper, &Unimock::new(())).await });

    send_line(&agent, b"CIPROXY-VM 2 STATE\n", None)
        .await
        .expect("send");
    let (line, fd) = receive_line(&agent).await.expect("answer");

    served.await.expect("helper task").expect("helper answer");
    assert_eq!(line, "ERR unsupported\n");
    assert!(fd.is_none());
}

#[tokio::test]
async fn receive_refuses_a_long_line_and_a_second_line() {
    for bytes in [vec![b'A'; 200], b"OK\nOK\n".to_vec()] {
        let (writer, reader) = UnixStream::pair().expect("pair");
        send_line(&writer, &bytes, None).await.expect("send");
        assert!(receive_line(&reader).await.is_err());
    }
}

#[tokio::test]
async fn receive_refuses_an_early_end() {
    let (writer, reader) = UnixStream::pair().expect("pair");
    send_line(&writer, b"OK", None).await.expect("send");
    drop(writer);
    let error = receive_line(&reader).await.expect_err("early end");
    assert_eq!(error.kind(), std::io::ErrorKind::UnexpectedEof);
}
