//! Machine, admin, and job-proxy listeners on isolated ephemeral ports.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use ciproxy_cache_sidecar::clock::Clock;
use ciproxy_cache_sidecar::machine::MachineCache;
use ciproxy_cache_sidecar::proxy::{ProxyConfig, VmProxy};
use ciproxy_cache_sidecar::service::{Service, admin_router, job_router};
use ciproxy_cache_sidecar::session::{Permissions, Registration, SessionCredential};
use ciproxy_cache_sidecar::storage::disk::DiskStorage;
use ciproxy_cache_sidecar::storage::interface::Storage;
use ciproxy_cache_sidecar::transport::{HttpTransport, Transport};
use ciproxy_protocol::cache::MachineCacheReport;
use ciproxy_protocol::identifiers::{GitRef, JobEvent, JobId, OrganizationId, RepositoryId};
use ciproxy_protocol::jobs::{JobFacts, Repository};
use ciproxy_protocol::secret::Secret;
use reqwest::Client;
use serde_json::{Value, json};
use tempfile::TempDir;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use super::github::Github;
use super::job::Job;

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
    github: Github,
    pub client: Client,
    pub machine: String,
    pub admin: String,
    pub clock: Arc<Time>,
    tasks: Vec<JoinHandle<()>>,
    network: Arc<dyn Transport>,
    capacity: AtomicU64,
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
        };
        fixture.restart().await;
        fixture
    }

    pub async fn restart(&mut self) {
        for task in self.tasks.drain(..) {
            task.abort();
        }
        let store = Arc::new(DiskStorage::new(
            self.root.path().join("cache"),
            self.capacity.load(Ordering::SeqCst),
        ));
        let _ = store.recover().await;
        let service: Arc<dyn Service> = Arc::new(MachineCache::new(
            store,
            self.network.clone(),
            self.clock.clone(),
            "admin-secret".into(),
        ));
        let jobs = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let admin = TcpListener::bind("127.0.0.1:0").await.unwrap();
        self.machine = format!("http://{}", jobs.local_addr().unwrap());
        self.admin = format!("http://{}", admin.local_addr().unwrap());
        let router = job_router(service.clone());
        self.tasks.push(tokio::spawn(async move {
            axum::serve(jobs, router).await.unwrap();
        }));
        self.tasks.push(tokio::spawn(async move {
            axum::serve(admin, admin_router(service)).await.unwrap();
        }));
    }

    pub async fn job(&self, id: &str, facts: JobFacts) -> Job {
        self.job_with(
            id,
            facts,
            Permissions {
                read: true,
                write: true,
            },
            false,
        )
        .await
    }

    pub async fn job_with(
        &self,
        id: &str,
        facts: JobFacts,
        permissions: Permissions,
        machine_down: bool,
    ) -> Job {
        let machine = if machine_down {
            "http://127.0.0.1:1"
        } else {
            &self.machine
        };
        self.job_via(id, facts, permissions, machine).await
    }

    pub async fn job_via(
        &self,
        id: &str,
        facts: JobFacts,
        permissions: Permissions,
        machine_url: &str,
    ) -> Job {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let registration = Registration {
            job_id: JobId(id.into()),
            facts,
            permissions,
            github_token: Secret::new(format!("github-{id}")),
            cache_url: Secret::new(format!("{}/legacy-prefix/", self.github.origin)),
            results_url: Secret::new(&self.github.origin),
            proxy_url: origin.clone(),
        };
        let credential: SessionCredential = self
            .client
            .post(format!("{}/sessions", self.admin))
            .bearer_auth("admin-secret")
            .json(&registration)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let config = ProxyConfig {
            machine_url: Secret::new(machine_url),
            machine_token: credential.token.clone(),
            registration: registration.clone(),
        };
        let proxy: Arc<dyn Service> = Arc::new(VmProxy::new(config, self.network.clone()));
        let task = tokio::spawn(async move {
            axum::serve(listener, job_router(proxy)).await.unwrap();
        });
        Job {
            origin,
            client: self.client.clone(),
            token: credential.token,
            task,
        }
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

pub fn facts(reference: &str) -> JobFacts {
    JobFacts {
        organization_id: OrganizationId("1".into()),
        repository: Repository {
            id: RepositoryId("10".into()),
            full_name: "org/repo".into(),
        },
        git_ref: GitRef(reference.into()),
        event: JobEvent("push".into()),
        head_repository: None,
        default_ref: GitRef("refs/heads/main".into()),
        base_ref: None,
    }
}
