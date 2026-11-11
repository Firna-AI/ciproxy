//! The real executable runs with isolated settings and bounded child waits.

use std::net::SocketAddr;
use std::path::Path;
use std::process::{ExitStatus, Stdio};
use std::time::Duration;

use tokio::process::{Child, Command};
use tokio::time::timeout;

/// Apply only explicit local inputs to the production executable.
pub(super) fn spawn(
    directory: &Path,
    origin: SocketAddr,
    proxy: SocketAddr,
    database_url: &str,
) -> Child {
    Command::new(env!("CARGO_BIN_EXE_ciproxy-control-plane"))
        .kill_on_drop(true)
        .env_clear()
        .envs([
            ("HTTP_BIND", "127.0.0.1:0".to_owned()),
            ("DATABASE_URL", database_url.to_owned()),
            ("PUBLIC_URL", "https://startup.invalid".to_owned()),
            ("IMAGE_URL_PREFIX", format!("http://{origin}/images")),
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
            ("HTTPS_PROXY", format!("http://{proxy}")),
            ("https_proxy", format!("http://{proxy}")),
            ("RUST_LOG", "off".to_owned()),
            ("TZ", "UTC".to_owned()),
            ("LANG", "C".to_owned()),
            ("LC_ALL", "C".to_owned()),
        ])
        .env("SECRETS_DIRECTORY", directory)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
}

/// Send the real Unix termination signal to the child process.
pub(crate) async fn terminate(child: &Child) -> ExitStatus {
    Command::new("kill")
        .arg("-TERM")
        .arg(child.id().unwrap().to_string())
        .status()
        .await
        .unwrap()
}

/// Keep the existing 15-second guard around the child exit wait.
pub(crate) async fn exit_code(child: &mut Child) -> Option<i32> {
    timeout(Duration::from_secs(15), child.wait())
        .await
        .ok()
        .and_then(Result::ok)
        .and_then(|status| status.code())
}
