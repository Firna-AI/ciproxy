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
    assert_exit_with_unreadable_usage("[]").await;
}

#[tokio::test]
async fn guest_reader_keeps_exit_when_usage_exceeds_the_json_depth_limit() {
    let deep = format!("{}0{}", "[".repeat(200), "]".repeat(200));
    assert_exit_with_unreadable_usage(&format!(r#"{{"cpu_milli_vcpu":{deep}}}"#)).await;
}

async fn assert_exit_with_unreadable_usage(usage: &str) {
    let (agent, mut guest) = UnixStream::pair().unwrap();
    let mut socket = framed(agent);
    let line = format!(
        "{{\"type\":\"runner_exited\",\"runner_id\":\"runner\",\"exit\":{{\"code\":7}},\"usage\":{usage}}}\n"
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
