//! The real executable cancels release discovery on a termination signal.

use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::http::StatusCode;
use axum::routing::get;
use tokio::net::TcpListener;
use tokio::process::Command;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use uuid::Uuid;

struct Fixture {
    directory: PathBuf,
    server: JoinHandle<()>,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.server.abort();
        std::fs::remove_dir_all(&self.directory).unwrap();
    }
}

#[tokio::test]
async fn sigterm_cancels_pending_image_discovery_and_exits_cleanly() {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.context/ciproxy-mvp/startup-shutdown")
        .join(Uuid::new_v4().simple().to_string());
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(
        directory.join("cdn-signing-key"),
        "AAECAwQFBgcICQoLDA0ODw==",
    )
    .unwrap();
    let entered = Arc::new(Notify::new());
    let observed = entered.clone();
    let routes = Router::new().route(
        "/images/release/manifest.json",
        get(move || {
            let observed = observed.clone();
            async move {
                observed.notify_one();
                std::future::pending::<(StatusCode, &'static str)>().await
            }
        }),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let fixture = Fixture {
        directory,
        server: tokio::spawn(async move {
            axum::serve(listener, routes).await.unwrap();
        }),
    };
    let mut child = Command::new(env!("CARGO_BIN_EXE_ciproxy-control-plane"))
        .kill_on_drop(true)
        .env_clear()
        .envs([
            ("HTTP_BIND", "127.0.0.1:0".to_owned()),
            ("DATABASE_URL", "postgresql://127.0.0.1/unused".to_owned()),
            ("PUBLIC_URL", "https://startup.invalid".to_owned()),
            ("IMAGE_URL_PREFIX", format!("http://{address}/images")),
            (
                "JOB_IMAGE_MANIFEST_PATH",
                "release/manifest.json".to_owned(),
            ),
            ("BOAT_KMS_KEY", "unused-startup-key".to_owned()),
            ("CDN_SIGNING_KEY_NAME", "startup-test".to_owned()),
            ("GITHUB_APP_ID", "1".to_owned()),
            ("GITHUB_OAUTH_CLIENT_ID", "unused-client".to_owned()),
            ("GITHUB_APP_SLUG", "unused-app".to_owned()),
            ("DOWNLOAD_URL", "https://startup.invalid/install".to_owned()),
            ("BUILD_ID", "startup-test".to_owned()),
            ("RUST_LOG", "off".to_owned()),
        ])
        .env("SECRETS_DIRECTORY", &fixture.directory)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    timeout(Duration::from_secs(15), entered.notified())
        .await
        .expect("the real executable must request its selected manifest");
    let signal = Command::new("kill")
        .arg("-TERM")
        .arg(child.id().unwrap().to_string())
        .status()
        .await
        .unwrap();
    assert_eq!(signal.code(), Some(0));
    let outcome = timeout(Duration::from_secs(15), child.wait()).await;
    assert_eq!(
        outcome
            .ok()
            .and_then(Result::ok)
            .and_then(|status| status.code()),
        Some(0),
        "SIGTERM must cancel the pending release and preserve a successful exit"
    );
}
