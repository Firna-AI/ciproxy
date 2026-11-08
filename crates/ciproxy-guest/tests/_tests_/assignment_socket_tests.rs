//! Source-hook wire integration through the production Unix socket boundary.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use ciproxy_protocol::identifiers::JobId;
use ciproxy_protocol::runtime::CachePermissions;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

use crate::assignment::{AssignmentSource, SocketAssignments};
use crate::guest_setup::PreloadedDocker;

const JOB_ID: &str = "00000000-0000-0000-0000-000000000123";
const PROXY_URL: &str = "http://172.17.0.1:8230";
const HANDOFF: &[u8] = concat!(
    "{\"handoff_protocol_version\":2,",
    "\"job_id\":\"00000000-0000-0000-0000-000000000123\",",
    "\"runtime\":{\"docker_data_bytes\":0,\"proxy_url\":\"\",",
    "\"github_token\":\"original-runtime-token\",",
    "\"cache_url\":\"https://cache.example/\",",
    "\"results_url\":\"https://results.example/\",",
    "\"permissions\":{\"read\":true,\"write\":true}}}\n"
)
.as_bytes();
const ACKNOWLEDGED: &[u8] = concat!(
    "{\"handoff_protocol_version\":2,",
    "\"job_id\":\"00000000-0000-0000-0000-000000000123\",",
    "\"acknowledged\":true}\n"
)
.as_bytes();

struct SocketFixture {
    directory: PathBuf,
}

impl SocketFixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let directory = std::env::temp_dir().join(format!(
            "ciproxy-worker-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&directory).unwrap();
        Self { directory }
    }

    fn path(&self) -> PathBuf {
        self.directory.join("assignment.sock")
    }
}

impl Drop for SocketFixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.directory).unwrap();
    }
}

#[tokio::test]
async fn source_hook_socket_preserves_credentials_and_returns_host_decisions() {
    for proxy_url in [Some(PROXY_URL), None] {
        let fixture = SocketFixture::new();
        let mut source = SocketAssignments {
            listener: UnixListener::bind(fixture.path()).unwrap(),
            preloaded: PreloadedDocker {
                proxy_url: PROXY_URL.to_owned(),
                data_bytes: 123_456,
            },
        };
        let worker = async {
            let stream = UnixStream::connect(fixture.path()).await.unwrap();
            let mut stream = BufReader::new(stream);
            stream.get_mut().write_all(HANDOFF).await.unwrap();
            let mut line = String::new();
            stream.read_line(&mut line).await.unwrap();
            assert_eq!(
                serde_json::from_str::<Value>(&line).unwrap(),
                json!({"handoff_protocol_version": 2, "job_id": JOB_ID, "preparing": true})
            );
            stream.get_mut().write_all(ACKNOWLEDGED).await.unwrap();
            line.clear();
            stream.read_line(&mut line).await.unwrap();
            let expected = match proxy_url {
                Some(_) => json!({
                    "handoff_protocol_version": 2,
                    "job_id": JOB_ID,
                    "decision": "cache_ready",
                    "proxy_url": "http://172.17.0.1:8230"
                }),
                None => json!({
                    "handoff_protocol_version": 2,
                    "job_id": JOB_ID,
                    "decision": "github_only"
                }),
            };
            assert_eq!(serde_json::from_str::<Value>(&line).unwrap(), expected);
        };
        let guest = async {
            let mut assignment = source.receive().await.unwrap();
            assert_eq!(assignment.handoff.handoff_protocol_version, 2);
            assert_eq!(assignment.handoff.job_id, JobId(JOB_ID.to_owned()));
            let runtime = &assignment.handoff.runtime;
            assert_eq!(
                runtime.github_token.expose_secret(),
                "original-runtime-token"
            );
            assert_eq!(runtime.cache_url.expose_secret(), "https://cache.example/");
            assert_eq!(
                runtime.results_url.expose_secret(),
                "https://results.example/"
            );
            assert_eq!(
                runtime.permissions,
                CachePermissions {
                    read: true,
                    write: true
                }
            );
            assert_eq!(runtime.proxy_url, "http://172.17.0.1:8230");
            assert_eq!(runtime.docker_data_bytes, 123_456);
            assert!(assignment.reply.begin().await.unwrap());
            assignment.reply.ready(proxy_url).await.unwrap();
        };
        tokio::join!(worker, guest);
    }
}
