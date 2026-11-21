//! An App-JWT installation peer with explicit barriers for proof refresh.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use axum::extract::OriginalUri;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::get;
use axum::{Json, Router};
use ciproxy_github::auth::AppJwt;
use ciproxy_github::installation_state::{AppInstallationClient, InstallationApi};
use reqwest::{Client, Url};
use secrecy::SecretString;
use serde_json::json;
use tokio::net::TcpListener;
use tokio::sync::{Notify, Semaphore};
use tokio::task::JoinHandle;

struct Signer;
impl AppJwt for Signer {
    fn token(&self) -> ciproxy_github::error::Result<SecretString> {
        Ok("fixture-app-jwt".into())
    }
}

pub(super) struct Http {
    pub(super) api: Arc<dyn InstallationApi>,
    pub(super) requests: Arc<Mutex<Vec<(String, bool)>>>,
    pub(super) status: Arc<Mutex<StatusCode>>,
    pub(super) hold: Arc<AtomicBool>,
    pub(super) entered: Arc<Notify>,
    pub(super) release: Arc<Semaphore>,
    task: JoinHandle<()>,
}

impl Drop for Http {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl Http {
    pub(super) async fn start(status: StatusCode) -> Self {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let status = Arc::new(Mutex::new(status));
        let hold = Arc::new(AtomicBool::new(false));
        let entered = Arc::new(Notify::new());
        let release = Arc::new(Semaphore::new(0));
        let observed = requests.clone();
        let response = status.clone();
        let held = hold.clone();
        let started = entered.clone();
        let available = release.clone();
        let router = Router::new().route(
            "/app/installations/{id}",
            get(move |OriginalUri(uri): OriginalUri, headers: HeaderMap| {
                let observed = observed.clone();
                let response = response.clone();
                let held = held.clone();
                let started = started.clone();
                let available = available.clone();
                async move {
                    observed.lock().unwrap().push((
                        uri.path().to_owned(),
                        headers
                            .get("authorization")
                            .is_some_and(|value| value == "Bearer fixture-app-jwt"),
                    ));
                    started.notify_one();
                    if held.load(Ordering::SeqCst) {
                        available.acquire().await.unwrap().forget();
                    }
                    (
                        *response.lock().unwrap(),
                        Json(json!({
                            "id": 43,
                            "account": {"id": 42, "login": "own-runtime", "type": "Organization"},
                            "suspended_at": null
                        })),
                    )
                }
            }),
        );
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        let api = Arc::new(AppInstallationClient::new(
            Client::new(),
            Url::parse(&format!("http://{address}/")).unwrap(),
            Arc::new(Signer),
        ));
        Self {
            api,
            requests,
            status,
            hold,
            entered,
            release,
            task,
        }
    }
}
