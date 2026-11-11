//! Machine, admin, and job-proxy listeners on isolated ephemeral ports.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use axum::body::{Body, to_bytes};
use axum::extract::State;
use axum::http::{Request, Response};
use axum::middleware::{self, Next};
use ciproxy_cache_sidecar::clock::Clock;
use ciproxy_cache_sidecar::machine::MachineCache;
use ciproxy_cache_sidecar::runtime_token::TokenVerifier;
use ciproxy_cache_sidecar::runtime_token::keys::CachedKeys;
use ciproxy_cache_sidecar::runtime_token::source::{HttpKeys, VerificationConfig};
use ciproxy_cache_sidecar::service::{Service, admin_router, job_router};
use ciproxy_cache_sidecar::storage::disk::DiskStorage;
use ciproxy_cache_sidecar::storage::interface::Storage;
use ciproxy_cache_sidecar::transport::{HttpTransport, Transport};
use ciproxy_protocol::cache::MachineCacheReport;
use reqwest::Client;
use serde_json::{Value, json};
use tempfile::TempDir;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use super::github::Github;
use super::tokens::serve_keys;

pub struct Time(pub AtomicU64);
impl Clock for Time {
    fn now(&self) -> u64 {
        self.0.load(Ordering::SeqCst)
    }
}
impl Time {
    pub fn advance(&self, seconds: u64) {
        self.0.fetch_add(seconds, Ordering::SeqCst);
    }
}

pub struct Fixture {
    pub root: TempDir,
    pub(super) github: Github,
    pub client: Client,
    pub machine: String,
    pub admin: String,
    pub clock: Arc<Time>,
    tasks: Vec<JoinHandle<()>>,
    pub(super) network: Arc<dyn Transport>,
    capacity: AtomicU64,
    pub admin_bodies: Arc<Mutex<Vec<Vec<u8>>>>,
}

impl Fixture {
    pub async fn new() -> Self {
        Self::with_disk(false, 1024 * 1024).await
    }

    pub async fn with_disk(broken: bool, capacity: u64) -> Self {
        let root = TempDir::new().unwrap();
        if broken {
            std::fs::write(root.path().join("cache"), b"not a directory").unwrap();
        }
        let mut fixture = Self {
            root,
            github: Github::start(),
            client: Client::new(),
            machine: String::new(),
            admin: String::new(),
            clock: Arc::new(Time(AtomicU64::new(1_000_000))),
            tasks: Vec::new(),
            network: Arc::new(HttpTransport::new().unwrap()),
            capacity: AtomicU64::new(capacity),
            admin_bodies: Arc::new(Mutex::new(Vec::new())),
        };
        fixture.restart().await;
        fixture
    }

    pub async fn restart(&mut self) {
        self.restart_budget(Some(self.capacity.load(Ordering::SeqCst)))
            .await;
    }

    pub async fn restart_waiting_for_budget(&mut self) {
        self.restart_budget(None).await;
    }

    async fn restart_budget(&mut self, budget: Option<u64>) {
        for task in self.tasks.drain(..) {
            task.abort();
        }
        let root = self.root.path().join("cache");
        let store = Arc::new(match budget {
            Some(bytes) => DiskStorage::new(root, bytes),
            None => DiskStorage::waiting_for_budget(root),
        });
        let _ = store.recover().await;
        let (jwks_url, task) = serve_keys().await;
        self.tasks.push(task);
        let source = Arc::new(HttpKeys::configured(
            self.network.clone(),
            VerificationConfig { jwks_url },
        ));
        let keys = Arc::new(CachedKeys::new(source, self.clock.clone()));
        let tokens = Arc::new(TokenVerifier::new(keys, self.clock.clone()));
        let service: Arc<dyn Service> = Arc::new(MachineCache::new(
            store,
            self.network.clone(),
            self.clock.clone(),
            "admin-secret".into(),
            tokens,
        ));
        let jobs = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let admin = TcpListener::bind("127.0.0.1:0").await.unwrap();
        self.machine = format!("http://{}", jobs.local_addr().unwrap());
        self.admin = format!("http://{}", admin.local_addr().unwrap());
        let router = job_router(service.clone());
        self.tasks.push(tokio::spawn(async move {
            axum::serve(jobs, router).await.unwrap();
        }));
        let admin_router = admin_router(service).layer(middleware::from_fn_with_state(
            self.admin_bodies.clone(),
            record_admin,
        ));
        self.tasks.push(tokio::spawn(async move {
            axum::serve(admin, admin_router).await.unwrap();
        }));
    }

    pub async fn entries(&self) -> Vec<ciproxy_cache_sidecar::storage::interface::Entry> {
        DiskStorage::new(
            self.root.path().join("cache"),
            self.capacity.load(Ordering::SeqCst),
        )
        .entries()
        .await
        .unwrap()
    }

    pub async fn report(&self) -> MachineCacheReport {
        self.client
            .get(format!("{}/report", self.admin))
            .bearer_auth("admin-secret")
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }

    pub async fn options(&self, options: Value) {
        self.client
            .post(format!("{}/_fixture/options", self.github.origin))
            .json(&options)
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap();
    }

    pub async fn wait_upload(&self, key: &str, bytes: u64) {
        let mut url =
            url::Url::parse(&format!("{}/_fixture/wait-upload", self.github.origin)).unwrap();
        url.query_pairs_mut()
            .append_pair("key", key)
            .append_pair("bytes", &bytes.to_string());
        assert_eq!(self.client.get(url).send().await.unwrap().status(), 200);
    }

    pub async fn seed(&self, key: &str, version: &str, base64: &str) {
        self.client
            .post(format!("{}/_fixture/seed", self.github.origin))
            .json(&json!({"key":key,"version":version,"base64":base64}))
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap();
    }

    pub async fn upstream(&self) -> Value {
        self.client
            .get(format!("{}/_fixture/state", self.github.origin))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }

    pub async fn delete_upstream(&self) {
        self.client
            .delete(format!("{}/_fixture/entries", self.github.origin))
            .send()
            .await
            .unwrap();
    }

    pub async fn clear(&self) {
        assert_eq!(
            self.client
                .post(format!("{}/clear", self.admin))
                .bearer_auth("admin-secret")
                .send()
                .await
                .unwrap()
                .status(),
            204
        );
    }

    pub async fn budget(&self, capacity: u64) {
        self.capacity.store(capacity, Ordering::SeqCst);
        assert_eq!(
            self.client
                .put(format!("{}/budget", self.admin))
                .bearer_auth("admin-secret")
                .json(&json!({"capacity_bytes":capacity}))
                .send()
                .await
                .unwrap()
                .status(),
            204
        );
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        for task in &self.tasks {
            task.abort();
        }
    }
}

async fn record_admin(
    State(bodies): State<Arc<Mutex<Vec<Vec<u8>>>>>,
    request: Request<Body>,
    next: Next,
) -> Response<Body> {
    let (parts, body) = request.into_parts();
    let bytes = to_bytes(body, 16 * 1024 * 1024).await.unwrap();
    bodies.lock().await.push(bytes.to_vec());
    next.run(Request::from_parts(parts, Body::from(bytes)))
        .await
}
