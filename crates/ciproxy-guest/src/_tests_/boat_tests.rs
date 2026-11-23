//! Sample delivery through the Boat HTTP sender to a loopback receiver.

use std::collections::HashMap;
use std::net::Ipv4Addr;

use ciproxy_protocol::guest::GuestToAgent;
use ciproxy_protocol::identifiers::RunnerId;
use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::secret::Secret;
use ciproxy_protocol::usage::{JobStep, JobStepTiming, JobUsage};
use reqwest::Client;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

use crate::usage::Reporter;

use super::BoatCallback;

const CALLBACK_PATH: &str = "/hooks/boat/runner-exit";

#[tokio::test]
async fn boat_sample_sender_posts_the_authorized_usage_report() {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
    let address = listener.local_addr().unwrap();
    let token = "loopback-test-token";
    let callback = BoatCallback {
        report_url: Secret::new(format!("http://{address}{CALLBACK_PATH}")),
        token: Secret::new(token),
        client: Client::builder().no_proxy().build().unwrap(),
    };
    let usage = JobUsage {
        platform: Platform::LinuxX64,
        first_point_at_ms: 1_700_000_000_000,
        point_seconds: 1,
        cpu_milli_vcpu: vec![1250],
        peak_memory_mib: vec![600],
        oom_kills: Some(vec![0]),
        steps: vec![JobStep {
            name: "Build".into(),
            timing: JobStepTiming::Skipped,
        }],
    };
    let mut sending = callback.post_samples(GuestToAgent::UsageSamples {
        runner_id: RunnerId("fixture-runner".into()),
        usage: usage.into(),
    });
    let (request_line, headers, body) = tokio::select! {
        biased;
        request = receive(listener) => Some(request),
        _ = &mut sending => None,
    }
    .expect("sample sender finished before the callback received a request");
    sending.await.unwrap();

    assert_eq!(
        request_line.trim_end(),
        format!("POST {CALLBACK_PATH} HTTP/1.1")
    );
    assert_eq!(
        headers.get("authorization"),
        Some(&format!("Bearer {token}"))
    );
    assert_eq!(
        body,
        json!({
            "type": "usage_samples",
            "runner_id": "fixture-runner",
            "usage": {
                "platform": "linux_x64", "first_point_at_ms": 1_700_000_000_000_i64,
                "point_seconds": 1, "cpu_milli_vcpu": [1250], "peak_memory_mib": [600],
                "oom_kills": [0], "steps": [{ "name": "Build", "status": "skipped" }],
            },
        })
    );
}

async fn receive(listener: TcpListener) -> (String, HashMap<String, String>, Value) {
    let (stream, _) = listener.accept().await.unwrap();
    let mut stream = BufReader::new(stream);
    let mut request_line = String::new();
    stream.read_line(&mut request_line).await.unwrap();
    let mut headers = HashMap::new();
    loop {
        let mut line = String::new();
        assert_ne!(
            stream.read_line(&mut line).await.unwrap(),
            0,
            "incomplete headers"
        );
        if line == "\r\n" {
            break;
        }
        let (name, value) = line.split_once(':').unwrap();
        headers.insert(name.to_ascii_lowercase(), value.trim().to_owned());
    }
    let mut body = vec![0; headers["content-length"].parse().unwrap()];
    stream.read_exact(&mut body).await.unwrap();
    stream
        .get_mut()
        .write_all(b"HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n")
        .await
        .unwrap();
    (
        request_line,
        headers,
        serde_json::from_slice(&body).unwrap(),
    )
}
