//! A real HTTP hop that can lose and recover its machine connection.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use axum::Router;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, Response, StatusCode};
use ciproxy_cache_sidecar::transport::{HttpTransport, Outbound, Transport};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

struct Hop {
    connected: Arc<AtomicBool>,
    destination: String,
    network: Arc<dyn Transport>,
}

pub struct Relay {
    pub origin: String,
    connected: Arc<AtomicBool>,
    task: JoinHandle<()>,
}

impl Relay {
    pub async fn start(destination: String) -> Self {
        let connected = Arc::new(AtomicBool::new(false));
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let state = Arc::new(Hop {
            connected: connected.clone(),
            destination,
            network: Arc::new(HttpTransport::new().unwrap()),
        });
        let task = tokio::spawn(async move {
            axum::serve(listener, Router::new().fallback(forward).with_state(state))
                .await
                .unwrap();
        });
        Self {
            origin,
            connected,
            task,
        }
    }

    pub fn reconnect(&self) {
        self.connected.store(true, Ordering::SeqCst);
    }
}

async fn forward(State(hop): State<Arc<Hop>>, request: Request<Body>) -> Response<Body> {
    if hop.connected.load(Ordering::SeqCst) {
        let (parts, body) = request.into_parts();
        return hop
            .network
            .send(Outbound {
                method: parts.method,
                url: format!("{}{}", hop.destination, parts.uri),
                headers: parts.headers,
                body,
            })
            .await
            .unwrap();
    }
    let mut response = Response::new(Body::empty());
    *response.status_mut() = StatusCode::SERVICE_UNAVAILABLE;
    response
}

impl Drop for Relay {
    fn drop(&mut self) {
        self.task.abort();
    }
}
