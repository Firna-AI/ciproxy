//! The real executable cancels startup and drains serving work on SIGTERM.

use std::io::ErrorKind;
use std::time::Duration;

use axum::http::StatusCode;
use tokio::time::timeout;

use self::startup_shutdown::database::Database;
use self::startup_shutdown::fixture::Fixture;
use self::startup_shutdown::http::{PendingWebhook, wait_until_ready, wait_until_refused};
use self::startup_shutdown::process::{exit_code, terminate};
use self::startup_shutdown::sockets::bound_address;

mod startup_shutdown;

#[tokio::test]
async fn sigterm_cancels_pending_image_discovery_and_exits_cleanly() {
    let (fixture, entered) = Fixture::pending().await;
    let mut child = fixture.spawn("postgresql://127.0.0.1/unused");
    timeout(Duration::from_secs(15), entered.notified())
        .await
        .expect("the real executable must request its selected manifest");
    assert_eq!(terminate(&child).await.code(), Some(0));
    assert_eq!(
        exit_code(&mut child).await,
        Some(0),
        "SIGTERM must cancel the pending release and preserve a successful exit"
    );
}

#[tokio::test]
async fn sigterm_stops_new_connections_and_drains_an_active_http_request() {
    timeout(Duration::from_secs(15), async {
        let database = Database::new();
        let fixture = Fixture::serving().await;
        let mut child = fixture.spawn(&database.url);
        let address = bound_address(&mut child).await;
        assert_eq!(
            wait_until_ready(address).await,
            Some(StatusCode::OK),
            "the real executable must finish its first reconciliation before SIGTERM"
        );
        let mut active = PendingWebhook::connect(address).await;
        assert_eq!(
            active.read_status().await,
            Ok(Some(StatusCode::CONTINUE)),
            "100 Continue must prove that the server started reading the request body"
        );
        active.send_partial_body().await;
        assert_eq!(terminate(&child).await.code(), Some(0));
        assert_eq!(
            wait_until_refused(address).await,
            Some(ErrorKind::ConnectionRefused),
            "SIGTERM must stop accepting connections while the request body is incomplete"
        );
        assert_eq!(
            active.complete().await,
            Ok(Some(StatusCode::BAD_REQUEST)),
            "the in-flight request must finish after SIGTERM and reject its missing GitHub event"
        );
        assert_eq!(
            exit_code(&mut child).await,
            Some(0),
            "the real executable must exit successfully after the HTTP drain"
        );
    })
    .await
    .expect("the serving shutdown test must not hang");
}
