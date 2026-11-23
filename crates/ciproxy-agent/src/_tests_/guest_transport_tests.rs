//! Guest frame decoding on a connected stream.

use ciproxy_protocol::guest::GuestToAgent;
use ciproxy_protocol::identifiers::RunnerId;
use ciproxy_protocol::runners::RunnerExit;
use ciproxy_protocol::usage::UsageReport;
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;

use super::framed;

#[tokio::test]
async fn guest_reader_keeps_exit_when_usage_does_not_decode() {
    let (agent, mut guest) = UnixStream::pair().unwrap();
    let mut socket = framed(agent);
    let line = concat!(
        r#"{"type":"runner_exited","runner_id":"runner","exit":{"code":7},"usage":[]}"#,
        "\n"
    );
    guest.write_all(line.as_bytes()).await.unwrap();
    assert_eq!(
        socket.receive().await.map_err(|error| error.to_string()),
        Ok(GuestToAgent::RunnerExited {
            runner_id: RunnerId("runner".into()),
            exit: RunnerExit { code: Some(7) },
            usage: Some(UsageReport::PointsDropped),
        })
    );
}
