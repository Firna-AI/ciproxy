//! Restartable production HTTP host for composed dashboard regressions.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use axum::Router;
use ciproxy_control_plane::http::dashboard_router;
use ciproxy_control_plane::pages::PageComposition;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use unimock::Unimock;

use super::fixture::Fixture;

pub(super) struct Server(pub(super) JoinHandle<()>);

impl Drop for Server {
    fn drop(&mut self) {
        self.0.abort();
    }
}

pub(super) async fn host(fixture: &Fixture, pages: &PageComposition) -> (String, Server) {
    let application = dashboard_router(
        Router::new(),
        fixture.state.clone(),
        pages.services.clone(),
        Arc::new(Unimock::new(())),
        Arc::new(AtomicBool::new(true)),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move { axum::serve(listener, application).await.unwrap() });
    (origin, Server(server))
}
