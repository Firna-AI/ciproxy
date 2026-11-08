//! Owned local origins and private files live until the child has stopped.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use tokio::net::TcpSocket;
use tokio::process::Child;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use uuid::Uuid;

use super::{cdn, process, secrets};

/// Startup inputs with an owned CDN listener and a reserved refused proxy.
pub(crate) struct Fixture {
    directory: PathBuf,
    origin: SocketAddr,
    proxy: TcpSocket,
    server: JoinHandle<()>,
}

impl Fixture {
    /// Hold discovery pending until the real executable receives SIGTERM.
    pub(crate) async fn pending() -> (Self, Arc<Notify>) {
        let (routes, entered) = cdn::pending();
        (Self::new(routes).await, entered)
    }

    /// Supply the release and mounted secrets that production startup reads.
    pub(crate) async fn serving() -> Self {
        let fixture = Self::new(cdn::released()).await;
        secrets::serving(&fixture.directory).await;
        fixture
    }

    async fn new(routes: Router) -> Self {
        let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../.context/control-plane-serving-shutdown")
            .join(Uuid::new_v4().simple().to_string());
        std::fs::create_dir_all(&directory).unwrap();
        secrets::cdn(&directory);
        let proxy = TcpSocket::new_v4().unwrap();
        proxy.bind("127.0.0.1:0".parse().unwrap()).unwrap();
        let (origin, server) = cdn::serve(routes).await;
        Self {
            directory,
            origin,
            proxy,
            server,
        }
    }

    /// Spawn with no inherited credentials, proxy bypasses, or service settings.
    ///
    /// The bound proxy socket never listens. It reserves its port and refuses
    /// HTTPS connections, so GitHub clients cannot use the public network.
    pub(crate) fn spawn(&self, database_url: &str) -> Child {
        process::spawn(
            &self.directory,
            self.origin,
            self.proxy.local_addr().unwrap(),
            database_url,
        )
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.server.abort();
        std::fs::remove_dir_all(&self.directory).unwrap();
    }
}
