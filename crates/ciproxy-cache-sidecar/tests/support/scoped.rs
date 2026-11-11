//! A test-owned scoped GitHub peer with trusted metadata and cache runtime routes.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use axum::body::{Body, to_bytes};
use axum::extract::State;
use axum::http::{Method, Request, Response, StatusCode};
use axum::response::IntoResponse;
use axum::{Json, Router};
use ciproxy_agent::cache_metadata::administration::{
    HttpMetadataAdministration, MetadataAdministration,
};
use ciproxy_agent::cache_metadata::client::RootMetadataClient;
use ciproxy_agent::cache_metadata::relay::{MetadataRelay, RootMetadataRelay, TokioRelayDelay};
use ciproxy_protocol::cache_metadata::CacheMetadataSource;
use ciproxy_protocol::secret::Secret;
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use super::Fixture;

#[derive(Clone)]
pub struct Record {
    pub reference: String,
    pub key: String,
    pub data: String,
    pub created: u64,
}

#[derive(Clone)]
struct Peer {
    records: Arc<Mutex<Vec<Record>>>,
    available: Arc<AtomicBool>,
    origin: String,
    scope_override: Arc<Mutex<Option<String>>>,
    incoming: Arc<Mutex<Option<Record>>>,
}

pub struct Scoped {
    peer: Peer,
    task: JoinHandle<()>,
    relay: Mutex<Option<JoinHandle<()>>>,
}

impl Scoped {
    pub async fn start(fixture: &mut Fixture) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        fixture.github.origin = origin.clone();
        let peer = Peer {
            records: Arc::new(Mutex::new(Vec::new())),
            available: Arc::new(AtomicBool::new(true)),
            origin,
            scope_override: Arc::new(Mutex::new(None)),
            incoming: Arc::new(Mutex::new(None)),
        };
        let router = Router::new().fallback(handle).with_state(peer.clone());
        let task = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
        let scoped = Self {
            peer,
            task,
            relay: Mutex::new(None),
        };
        scoped.configure(fixture).await;
        scoped
    }

    pub async fn configure(&self, fixture: &Fixture) {
        if let Some(task) = self.relay.lock().await.take() {
            task.abort();
        }
        let token_file = fixture.root.path().join("synthetic-admin-token");
        tokio::fs::write(&token_file, "admin-secret").await.unwrap();
        let administration = Arc::new(HttpMetadataAdministration::new(
            fixture.client.clone(),
            fixture.admin.clone(),
            token_file,
        ));
        administration.enable().await.unwrap();
        let relay: Arc<dyn MetadataRelay> = Arc::new(RootMetadataRelay {
            administration,
            hosted: Arc::new(RootMetadataClient::new(
                fixture.client.clone(),
                CacheMetadataSource {
                    url: Secret::new(format!("{}/metadata", self.peer.origin)),
                    credential: Secret::new("synthetic-machine-credential"),
                },
            )),
            delay: Arc::new(TokioRelayDelay),
        });
        *self.relay.lock().await = Some(tokio::spawn(async move {
            relay.run().await;
        }));
    }

    pub async fn add(&self, reference: &str, key: &str, data: &str, created: u64) {
        self.peer.records.lock().await.push(Record {
            reference: reference.into(),
            key: key.into(),
            data: data.into(),
            created,
        });
    }

    pub async fn remove(&self, reference: &str) {
        self.peer
            .records
            .lock()
            .await
            .retain(|record| record.reference != reference);
    }

    pub async fn override_scope(&self, reference: &str) {
        *self.peer.scope_override.lock().await = Some(reference.into());
    }

    pub async fn arrive_during_lookup(&self, reference: &str, key: &str, data: &str, created: u64) {
        *self.peer.incoming.lock().await = Some(Record {
            reference: reference.into(),
            key: key.into(),
            data: data.into(),
            created,
        });
    }

    pub fn available(&self) {
        self.peer.available.store(true, Ordering::SeqCst);
    }

    pub fn unavailable(&self) {
        self.peer.available.store(false, Ordering::SeqCst);
    }
}

impl Drop for Scoped {
    fn drop(&mut self) {
        self.task.abort();
        if let Some(task) = self.relay.get_mut().take() {
            task.abort();
        }
    }
}

async fn handle(State(peer): State<Peer>, request: Request<Body>) -> Response<Body> {
    let path = request.uri().path().to_owned();
    if path == "/metadata" {
        if request.headers().get("authorization").unwrap() != "Bearer synthetic-machine-credential"
        {
            return response(StatusCode::UNAUTHORIZED, Body::empty());
        }
        if !peer.available.load(Ordering::SeqCst) {
            return response(StatusCode::SERVICE_UNAVAILABLE, Body::empty());
        }
        let entries: Vec<_> = peer
            .records
            .lock()
            .await
            .iter()
            .map(|entry| {
                json!({
                    "id":entry.created.to_string(),"git_ref":entry.reference,
                    "key":entry.key,"version":"v1","created_at":entry.created.to_string()
                })
            })
            .collect();
        return json_response(json!({"entries":entries}));
    }
    if let Some(id) = path.strip_prefix("/archive/") {
        let records = peer.records.lock().await;
        let record = records
            .iter()
            .find(|entry| entry.created.to_string() == id)
            .unwrap();
        return response(StatusCode::OK, Body::from(record.data.clone()));
    }
    let token = request.headers()["authorization"]
        .to_str()
        .unwrap()
        .strip_prefix("Bearer ")
        .unwrap();
    let claims = jsonwebtoken::dangerous::insecure_decode::<Value>(token)
        .unwrap()
        .claims;
    let access: Vec<Value> = serde_json::from_str(claims["ac"].as_str().unwrap()).unwrap();
    let refs: Vec<_> = access
        .iter()
        .map(|scope| scope["Scope"].as_str().unwrap().to_owned())
        .collect();
    if let Some(incoming) = peer.incoming.lock().await.take() {
        peer.records.lock().await.push(incoming);
    }
    let legacy = request.method() == Method::GET;
    let (keys, version) = if legacy {
        let pairs: Vec<_> = url::form_urlencoded::parse(request.uri().query().unwrap().as_bytes())
            .into_owned()
            .collect();
        (
            pairs
                .iter()
                .find(|(key, _)| key == "keys")
                .unwrap()
                .1
                .split(',')
                .map(str::to_owned)
                .collect::<Vec<_>>(),
            pairs
                .iter()
                .find(|(key, _)| key == "version")
                .unwrap()
                .1
                .clone(),
        )
    } else {
        let bytes = to_bytes(request.into_body(), 1024 * 1024).await.unwrap();
        let query: Value = serde_json::from_slice(&bytes).unwrap();
        let mut keys = vec![query["key"].as_str().unwrap().to_owned()];
        if let Some(restore) = query["restoreKeys"].as_array() {
            keys.extend(restore.iter().map(|key| key.as_str().unwrap().to_owned()));
        }
        (keys, query["version"].as_str().unwrap().to_owned())
    };
    let records = peer.records.lock().await;
    let selected = refs.iter().find_map(|reference| {
        keys.iter().find_map(|key| {
            records
                .iter()
                .filter(|entry| entry.reference == *reference && version == "v1")
                .filter(|entry| entry.key.starts_with(key))
                .max_by_key(|entry| (entry.key == *key, entry.created))
        })
    });
    let optional_scope = peer.scope_override.lock().await.clone();
    match selected {
        Some(entry) if legacy => json_response(
            json!({"cacheKey":entry.key,"scope":optional_scope.as_deref().unwrap_or(&entry.reference),
            "archiveLocation":format!("{}/archive/{}",peer.origin,entry.created)}),
        ),
        Some(entry) => json_response(json!({"ok":true,"matchedKey":entry.key,
            "signedDownloadUrl":format!("{}/archive/{}",peer.origin,entry.created)})),
        None if legacy => response(StatusCode::NO_CONTENT, Body::empty()),
        None => json_response(json!({"ok":false})),
    }
}

fn json_response(value: Value) -> Response<Body> {
    Json(value).into_response()
}

fn response(status: StatusCode, body: Body) -> Response<Body> {
    Response::builder().status(status).body(body).unwrap()
}
