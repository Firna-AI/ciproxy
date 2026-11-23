//! The macOS release comes from presigned R2 reads and is optional at start.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::Router;
use axum::extract::OriginalUri;
use axum::http::StatusCode;
use ciproxy_control_plane::image_release::{ImageReleases, SignedReleases};
use ciproxy_control_plane::macos_release::{MacosImageSettings, read_macos_release};
use ciproxy_control_plane::presigned_signing::{PresignedSigning, StoreKey};
use ciproxy_github::runtime::RuntimeMock;
use ciproxy_protocol::identifiers::ImageId;
use ciproxy_protocol::images::ImageDownload;
use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::secret::Secret;
use tokio::net::{TcpListener, TcpSocket};
use unimock::{MockFn, Unimock, matching};

use self::tls_store::TlsStore;

mod loopback;
mod tls_store;

const RELEASE: &str = "/ciproxy-macos-images/releases/macos26-v1";
const KEY: &str = r#"{"access_key_id": "fixture-key-id", "secret_access_key": "fixture-secret"}"#;

/// An HTTPS store that answers each path with a fixed status and body, and records each request.
struct Store {
    tls: TlsStore,
    requests: Arc<Mutex<Vec<String>>>,
}

const MANIFEST: &str =
    r#"{"image_id": "macos26-v1", "root_disk_bytes": 214748364800, "free_job_bytes": 1}"#;

async fn store(status: StatusCode) -> Store {
    store_with(status, MANIFEST.to_owned()).await
}

async fn store_with(status: StatusCode, manifest: String) -> Store {
    let files: HashMap<String, String> = HashMap::from([
        (format!("{RELEASE}/manifest.json"), manifest),
        (
            format!("{RELEASE}/downloads.json"),
            r#"{"firecracker": {"file": "firecracker.tar.zst", "sha256": "linux", "size_bytes": 1},
                "macos": {"file": "macos.tar", "sha256": "macos-checksum", "size_bytes": 42949672960}}"#
                .to_owned(),
        ),
    ]);
    let requests = Arc::new(Mutex::new(Vec::new()));
    let recorded = requests.clone();
    let routes = Router::new().fallback(move |OriginalUri(uri): OriginalUri| {
        recorded.lock().unwrap().push(uri.to_string());
        let body = files.get(uri.path()).cloned();
        async move {
            match body {
                Some(body) if status == StatusCode::OK => (status, body),
                Some(_) => (status, String::new()),
                None => (StatusCode::NOT_FOUND, String::new()),
            }
        }
    });
    Store {
        tls: tls_store::serve(routes).await,
        requests,
    }
}

fn settings(store: &Store) -> MacosImageSettings {
    MacosImageSettings {
        url_prefix: Some(format!(
            "https://{}/ciproxy-macos-images",
            store.tls.address
        )),
        manifest_path: Some("releases/macos26-v1/manifest.json".to_owned()),
        key: Some(Secret::new(KEY)),
    }
}

/// Read with a clock that signs, or with one that panics when a read is signed.
async fn read(store: &Store, settings: &MacosImageSettings, signs: bool) -> Option<ImageDownload> {
    let clock = Arc::new(if signs {
        Unimock::new(
            RuntimeMock::now.each_call(matching!()).returns(
                "2026-11-05T12:00:00Z"
                    .parse::<chrono::DateTime<chrono::Utc>>()
                    .unwrap(),
            ),
        )
    } else {
        Unimock::new(())
    });
    read_macos_release(settings, client(&store.tls), clock).await
}

#[tokio::test]
async fn the_macos_artifact_is_read_with_presigned_urls_that_expire_after_five_minutes() {
    let store = store(StatusCode::OK).await;
    let download = read(&store, &settings(&store), true).await.unwrap();
    assert_eq!(download.image_id, ImageId("macos26-v1".to_owned()));
    assert_eq!(download.platform, Platform::MacosArm64);
    assert_eq!(download.sha256, "macos-checksum");
    assert_eq!(download.size_bytes, 42_949_672_960);
    assert_eq!(download.root_disk_bytes, 214_748_364_800);
    assert_eq!(
        download.download_url.expose_secret(),
        format!("https://{}{RELEASE}/macos.tar", store.tls.address)
    );
    let requests = store.requests.lock().unwrap().clone();
    let paths: Vec<&str> = requests
        .iter()
        .map(|request| request.split_once('?').unwrap().0)
        .collect();
    assert_eq!(
        paths,
        [
            format!("{RELEASE}/manifest.json"),
            format!("{RELEASE}/downloads.json")
        ]
    );
    for request in &requests {
        let query = request.split_once('?').unwrap().1;
        let (unsigned, signature) = query.split_once("&X-Amz-Signature=").unwrap();
        assert_eq!(
            unsigned,
            "X-Amz-Algorithm=AWS4-HMAC-SHA256\
             &X-Amz-Credential=fixture-key-id%2F20261105%2Fauto%2Fs3%2Faws4_request\
             &X-Amz-Date=20261105T120000Z&X-Amz-Expires=300&X-Amz-SignedHeaders=host"
        );
        assert_eq!(signature.len(), 64, "{request}");
    }
}

#[tokio::test]
async fn macos_images_stay_off_without_a_read_when_a_setting_is_missing_or_malformed() {
    let store = store(StatusCode::OK).await;
    let complete = settings(&store);
    let cases = [
        (
            "no prefix",
            MacosImageSettings {
                url_prefix: None,
                ..complete.clone()
            },
        ),
        (
            "no manifest path",
            MacosImageSettings {
                manifest_path: None,
                ..complete.clone()
            },
        ),
        (
            "no key",
            MacosImageSettings {
                key: None,
                ..complete.clone()
            },
        ),
        (
            "a key without its secret",
            MacosImageSettings {
                key: Some(Secret::new(r#"{"access_key_id": "fixture-key-id"}"#)),
                ..complete.clone()
            },
        ),
        (
            "a prefix without https",
            MacosImageSettings {
                url_prefix: Some(format!("http://{}/ciproxy-macos-images", store.tls.address)),
                ..complete.clone()
            },
        ),
        (
            "a key that is not JSON",
            MacosImageSettings {
                key: Some(Secret::new("fixture-secret")),
                ..complete
            },
        ),
    ];
    for (case, settings) in cases {
        assert_eq!(
            read(&store, &settings, false)
                .await
                .map(|download| download.image_id),
            None,
            "{case}"
        );
        assert_eq!(
            *store.requests.lock().unwrap(),
            Vec::<String>::new(),
            "{case}"
        );
    }
}

#[tokio::test]
async fn a_failed_release_read_turns_macos_images_off() {
    let refused = store(StatusCode::FORBIDDEN).await;
    let oversized = store_with(StatusCode::OK, " ".repeat(1024 * 1024 + 1)).await;
    for (case, store) in [("refused", refused), ("larger than 1 MiB", oversized)] {
        assert_eq!(
            read(&store, &settings(&store), true)
                .await
                .map(|download| download.image_id),
            None,
            "{case}"
        );
        assert_eq!(store.requests.lock().unwrap().len(), 1, "{case}");
    }
}

/// Read one release through the reader that both platforms share.
async fn read_release(prefix: String) -> String {
    let releases = SignedReleases {
        http: loopback::client().build().unwrap(),
        prefix,
        manifest_path: "releases/macos26-v1/manifest.json".to_owned(),
        signing: Arc::new(PresignedSigning::new(
            StoreKey::from_json("R2_IMAGES_KEY", KEY).unwrap(),
            "auto",
            Duration::from_secs(300),
        )),
        clock: Arc::new(Unimock::new(
            RuntimeMock::now.each_call(matching!()).returns(
                "2026-11-05T12:00:00Z"
                    .parse::<chrono::DateTime<chrono::Utc>>()
                    .unwrap(),
            ),
        )),
        platform: Platform::MacosArm64,
    };
    let error = releases.current().await.err().unwrap();
    format!("{error:?}")
}

#[tokio::test]
async fn failed_release_reads_keep_the_signed_url_out_of_the_error() {
    let (_reserved, closed) = refused();
    let unreadable = Router::new().fallback(|| async { (StatusCode::OK, "not json") });
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, unreadable).await.unwrap() });
    for (case, prefix, cause) in [
        (
            "refused connection",
            format!("http://{closed}/ciproxy-macos-images"),
            "error sending request",
        ),
        (
            "body that is not JSON",
            format!("http://{address}/ciproxy-macos-images"),
            "expected ident at line 1 column 2",
        ),
    ] {
        let error = read_release(prefix).await;
        assert!(error.contains(cause), "{case}: {error}");
        assert!(!error.contains("X-Amz-Signature"), "{case}: {error}");
        assert!(!error.contains("manifest.json"), "{case}: {error}");
    }
}
/// The production client policy, trusting this store and using no proxy.
fn client(store: &TlsStore) -> reqwest::Client {
    loopback::client()
        .add_root_certificate(reqwest::Certificate::from_pem(store.certificate.as_bytes()).unwrap())
        .build()
        .unwrap()
}

/// A bound socket that never listens, so its port refuses each connection.
///
/// Keep the socket for the whole test, so no other listener takes the port.
fn refused() -> (TcpSocket, SocketAddr) {
    let socket = TcpSocket::new_v4().unwrap();
    socket.bind("127.0.0.1:0".parse().unwrap()).unwrap();
    let address = socket.local_addr().unwrap();
    (socket, address)
}
