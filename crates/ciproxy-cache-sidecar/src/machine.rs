//! Authenticated machine service with one isolated session for each job VM.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum::body::{Body, to_bytes};
use axum::http::{HeaderMap, Method, Request, Response, StatusCode};
use ciproxy_protocol::jobs::JobScope;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::clock::Clock;
use crate::download::Download;
use crate::error::{CacheError, Result, ResultExt};
use crate::metrics::{Metrics, Reports};
use crate::runtime_token::RuntimeTokens;
use crate::scope::can_save;
use crate::service::Service;
use crate::session::Registration;
use crate::storage::interface::Storage;
use crate::transport::{Outbound, Transport};
use crate::upload::state::Upload;
use crate::upstream::{canonicalize, destination};
use crate::wire::{V2, cache_path, miss, save_path, status};

pub(crate) struct Session {
    pub registration: Registration,
    pub scope: JobScope,
    pub private_ref: String,
    pub uploads: Mutex<HashMap<String, Arc<Mutex<Upload>>>>,
    pub downloads: Mutex<HashMap<String, Arc<Mutex<Download>>>>,
}

/// One machine's cache boundary. Consume it through `dyn Service`.
pub struct MachineCache {
    pub(crate) store: Arc<dyn Storage>,
    pub(crate) network: Arc<dyn Transport>,
    pub(crate) clock: Arc<dyn Clock>,
    pub(crate) metrics: Arc<dyn Reports>,
    pub(crate) sessions: Mutex<HashMap<String, Arc<Session>>>,
    pub(crate) admin_token: String,
    pub(crate) tokens: Arc<dyn RuntimeTokens>,
}

impl MachineCache {
    /// Compose a service from injected machine IO and an agent-only secret.
    pub fn new(
        store: Arc<dyn Storage>,
        network: Arc<dyn Transport>,
        clock: Arc<dyn Clock>,
        admin_token: String,
        tokens: Arc<dyn RuntimeTokens>,
    ) -> Self {
        Self {
            metrics: Arc::new(Metrics::new(store.clone(), clock.clone())),
            store,
            network,
            clock,
            sessions: Mutex::new(HashMap::new()),
            admin_token,
            tokens,
        }
    }

    pub(crate) async fn session(&self, headers: &HeaderMap) -> Result<Arc<Session>> {
        let token = headers
            .get("x-ciproxy-session")
            .and_then(|header| header.to_str().ok())
            .ok_or(CacheError::Unauthorized)?;
        self.sessions
            .lock()
            .await
            .get(token)
            .cloned()
            .ok_or(CacheError::Unauthorized)
    }

    pub(crate) async fn upstream(
        &self,
        session: &Session,
        method: Method,
        path: &str,
        mut headers: HeaderMap,
        body: Body,
    ) -> Result<Response<Body>> {
        let registration = &session.registration;
        headers.remove("x-ciproxy-session");
        headers.remove("authorization");
        let authorization = format!("Bearer {}", registration.github_token.expose_secret());
        headers.insert("authorization", authorization.parse().internal()?);
        let url = destination(registration, path)?;
        self.network
            .send(Outbound {
                method,
                url,
                headers,
                body,
            })
            .await
    }

    pub(crate) async fn dispatch(
        &self,
        session: Arc<Session>,
        request: Request<Body>,
    ) -> Result<Response<Body>> {
        let (parts, body) = request.into_parts();
        let path = parts.uri.path();
        if path == "/ciproxy/fallback" {
            self.metrics.fallback(&session.registration.job_id).await;
            return Ok(status(StatusCode::NO_CONTENT));
        }
        if path.starts_with("/ciproxy/blob/") {
            return self
                .blob(session, parts.method, parts.uri, parts.headers, body)
                .await;
        }
        let full_path = parts
            .uri
            .path_and_query()
            .map_or(path, |value| value.as_str());
        if !cache_path(path) {
            return self
                .upstream(&session, parts.method, full_path, parts.headers, body)
                .await;
        }
        if save_path(path, &parts.method)
            && !can_save(
                &session.registration.facts,
                &session.scope,
                session.registration.permissions.write,
            )
        {
            return Ok(miss(path));
        }
        if parts.method == Method::GET && path.ends_with("/cache") {
            return self.lookup_legacy(session, parts.uri, parts.headers).await;
        }
        if parts.method == Method::PATCH {
            return self
                .upload_legacy(session, parts.uri, parts.headers, body)
                .await;
        }
        let bytes = to_bytes(body, 1024 * 1024).await.internal()?;
        if path == format!("{V2}GetCacheEntryDownloadURL") {
            self.lookup_v2(session, &bytes, parts.headers).await
        } else if path == format!("{V2}CreateCacheEntry")
            || path.ends_with("/caches") && parts.method == Method::POST
        {
            self.reserve(session, path, &bytes, parts.headers).await
        } else if path == format!("{V2}FinalizeCacheEntryUpload")
            || path.contains("/caches/") && parts.method == Method::POST
        {
            self.finalize(session, path, &bytes, parts.headers).await
        } else {
            self.upstream(
                &session,
                parts.method,
                full_path,
                parts.headers,
                Body::from(bytes),
            )
            .await
        }
    }
}

#[async_trait]
impl Service for MachineCache {
    async fn handle(&self, request: Request<Body>) -> Response<Body> {
        let request = match canonicalize(request) {
            Ok(request) => request,
            Err(_) => return status(StatusCode::NO_CONTENT),
        };
        let path = request.uri().path().to_owned();
        let session = match self.session(request.headers()).await {
            Ok(session) => session,
            Err(_) => return status(StatusCode::UNAUTHORIZED),
        };
        match self.dispatch(session.clone(), request).await {
            Ok(response) => response,
            Err(_) => {
                tracing::warn!("Machine cache operation unavailable");
                self.metrics.fallback(&session.registration.job_id).await;
                if cache_path(&path) || path.starts_with("/ciproxy/blob/") {
                    miss(&path)
                } else {
                    status(StatusCode::BAD_GATEWAY)
                }
            }
        }
    }

    async fn admin(&self, request: Request<Body>) -> Response<Body> {
        let authorization = request
            .headers()
            .get("authorization")
            .and_then(|header| header.to_str().ok());
        if authorization != Some(format!("Bearer {}", self.admin_token).as_str()) {
            return status(StatusCode::UNAUTHORIZED);
        }
        match self.administration(request).await {
            Ok(response) => response,
            Err(_) => status(StatusCode::SERVICE_UNAVAILABLE),
        }
    }
}

pub(crate) fn transfer_url(session: &Session, id: &str) -> String {
    format!(
        "{}/ciproxy/blob/{id}",
        session.registration.proxy_url.trim_end_matches('/')
    )
}

pub(crate) fn identifier() -> String {
    Uuid::new_v4().to_string()
}
