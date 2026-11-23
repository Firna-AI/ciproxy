//! Real signed CDN requests preserve release facts and never log signature bytes.

use std::io::Write;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::extract::OriginalUri;
use axum::http::StatusCode;
use axum::routing::get;
use ciproxy_control_plane::error::Error;
use ciproxy_control_plane::image_release::{ImageReleases, SignedReleases};
use ciproxy_control_plane::image_signing::CdnSigning;
use ciproxy_gateway::logging;
use ciproxy_github::runtime::RuntimeMock;
use ciproxy_protocol::platform::Platform;
use tokio::net::TcpListener;
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use unimock::{MockFn, Unimock, matching};

mod loopback;

#[derive(Clone)]
struct Capture(Arc<Mutex<Vec<u8>>>);
impl Write for Capture {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn signed_manifest_and_bundle_metadata_reads_keep_credentials_out_of_trace_logs() {
    let captured = Arc::new(Mutex::new(Vec::new()));
    let writer = Capture(captured.clone());
    let subscriber = logging::subscriber(
        Ok("trace".to_owned()),
        BoxMakeWriter::new(move || writer.clone()),
    );
    tracing::subscriber::set_global_default(subscriber.subscriber).unwrap();
    let urls = Arc::new(Mutex::new(Vec::new()));
    let manifest_urls = urls.clone();
    let downloads_urls = urls.clone();
    let routes=Router::new().route("/images/full/manifest.json",get(move|OriginalUri(uri):OriginalUri|{manifest_urls.lock().unwrap().push(uri.to_string());async{(StatusCode::OK,"{\"image_id\":\"full-release\",\"root_disk_bytes\":62008590336}")}}))
 .route("/images/full/downloads.json",get(move|OriginalUri(uri):OriginalUri|{downloads_urls.lock().unwrap().push(uri.to_string());async{(StatusCode::OK,"{\"firecracker\":{\"file\":\"firecracker.tar.zst\",\"sha256\":\"fixture-checksum\",\"size_bytes\":12345}}")}}));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, routes).await.unwrap();
    });
    let releases = SignedReleases {
        http: loopback::client().build().unwrap(),
        prefix: format!("http://{address}/images"),
        manifest_path: "full/manifest.json".to_owned(),
        signing: Arc::new(
            CdnSigning::new("test-image-key".to_owned(), "AAECAwQFBgcICQoLDA0ODw==").unwrap(),
        ),
        clock: Arc::new(Unimock::new(
            RuntimeMock::now.each_call(matching!()).returns(
                "2026-11-05T12:00:00Z"
                    .parse::<chrono::DateTime<chrono::Utc>>()
                    .unwrap(),
            ),
        )),
        platform: Platform::LinuxX64,
    };
    let release = releases.current().await.unwrap();
    assert_eq!(release.root_disk_bytes, 62_008_590_336);
    assert_eq!(release.download.image_id.0, "full-release");
    assert_eq!(release.download.sha256, "fixture-checksum");
    assert_eq!(release.download.size_bytes, 12345);
    assert_eq!(
        release.download.download_url.expose_secret(),
        format!("http://{address}/images/full/firecracker.tar.zst")
    );
    let urls = urls.lock().unwrap().clone();
    assert_eq!(urls.len(), 2);
    assert!(urls[0].starts_with(
        "/images/full/manifest.json?Expires=1793887200&KeyName=test-image-key&Signature="
    ));
    assert!(urls[1].starts_with(
        "/images/full/downloads.json?Expires=1793887200&KeyName=test-image-key&Signature="
    ));
    tracing::trace!(target:"ciproxy_control_plane::signed_image_test","unrelated diagnostic stays visible");
    server.abort();
    let _ = server.await;
    let output = String::from_utf8(captured.lock().unwrap().clone()).unwrap();
    assert!(output.contains("unrelated diagnostic stays visible"));
    for request in urls {
        let signature = request.split("&Signature=").nth(1).unwrap();
        assert!(
            !output.contains(signature),
            "signed image URL leaked to logs"
        );
    }
    assert!(!output.contains("AAECAwQFBgcICQoLDA0ODw=="));
}

/// Declare 2 MiB and close after one byte, so only the declared length is too large.
async fn declared_oversize_origin() -> std::net::SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        loop {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = [0; 4096];
            let _ = tokio::io::AsyncReadExt::read(&mut socket, &mut request).await;
            let head = b"HTTP/1.1 200 OK\r\nContent-Length: 2097152\r\n\r\n{";
            let _ = tokio::io::AsyncWriteExt::write_all(&mut socket, head).await;
        }
    });
    address
}

#[tokio::test]
async fn linux_release_metadata_over_one_mebibyte_is_a_start_error() {
    let streamed = || {
        let chunks = (0..128).map(|_| Ok::<_, std::io::Error>(vec![b' '; 64 * 1024]));
        axum::body::Body::from_stream(futures_util::stream::iter(chunks))
    };
    let routes = Router::new().route(
        "/images/streamed/manifest.json",
        get(move || async move { streamed() }),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, routes).await.unwrap() });
    let declared = declared_oversize_origin().await;
    for (case, origin) in [("declared", declared), ("streamed", address)] {
        let releases = SignedReleases {
            http: loopback::client().build().unwrap(),
            prefix: format!("http://{origin}/images"),
            manifest_path: format!("{case}/manifest.json"),
            signing: Arc::new(
                CdnSigning::new("test-image-key".to_owned(), "AAECAwQFBgcICQoLDA0ODw==").unwrap(),
            ),
            clock: Arc::new(Unimock::new(
                RuntimeMock::now.each_call(matching!()).returns(
                    "2026-11-05T12:00:00Z"
                        .parse::<chrono::DateTime<chrono::Utc>>()
                        .unwrap(),
                ),
            )),
            platform: Platform::LinuxX64,
        };
        let error = releases.current().await.err();
        assert!(
            matches!(error, Some(Error::ReleaseTooLarge)),
            "{case}: {error:?}"
        );
    }
}

#[tokio::test]
async fn a_repeated_linux_archive_entry_is_a_start_error() {
    let routes = Router::new()
        .route(
            "/images/full/manifest.json",
            get(|| async { r#"{"image_id":"full-release","root_disk_bytes":62008590336}"# }),
        )
        .route(
            "/images/full/downloads.json",
            get(|| async {
                r#"{"firecracker":{"file":"first.tar.zst","sha256":"first","size_bytes":1},
                    "firecracker":{"file":"second.tar.zst","sha256":"second","size_bytes":2}}"#
            }),
        );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, routes).await.unwrap() });
    let releases = SignedReleases {
        http: loopback::client().build().unwrap(),
        prefix: format!("http://{address}/images"),
        manifest_path: "full/manifest.json".to_owned(),
        signing: Arc::new(
            CdnSigning::new("test-image-key".to_owned(), "AAECAwQFBgcICQoLDA0ODw==").unwrap(),
        ),
        clock: Arc::new(Unimock::new(
            RuntimeMock::now.each_call(matching!()).returns(
                "2026-11-05T12:00:00Z"
                    .parse::<chrono::DateTime<chrono::Utc>>()
                    .unwrap(),
            ),
        )),
        platform: Platform::LinuxX64,
    };
    let error = format!("{:?}", releases.current().await.err());
    assert!(error.contains("duplicate field `firecracker`"), "{error}");
}
