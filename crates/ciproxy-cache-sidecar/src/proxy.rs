//! VM cache proxy with safe outage misses and direct results passthrough.

use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{HeaderMap, Method, Request, Response, StatusCode};
use ciproxy_protocol::cache::CacheConnection;
use ciproxy_protocol::runtime::CacheProxyConfig;
use tokio::sync::RwLock;

use crate::error::{Result, ResultExt};
use crate::service::Service;
use crate::transport::{Outbound, Transport};
use crate::upstream::{canonicalize, destination};
use crate::wire::{cache_path, miss, save_path, status};

/// VM HTTP forwarding boundary. Consume it through `dyn Service`.
pub struct VmProxy {
    config: CacheProxyConfig,
    connection: RwLock<CacheConnection>,
    network: Arc<dyn Transport>,
}

/// Private parent-pipe updates never change the original assignment authority.
#[async_trait]
pub trait ProxyControl: Service {
    /// Replace only the machine endpoint and its narrow session credential.
    async fn replace_connection(&self, connection: CacheConnection);
}

impl VmProxy {
    /// Compose a proxy from trusted assignment data and injected HTTP IO.
    pub fn new(config: CacheProxyConfig, network: Arc<dyn Transport>) -> Self {
        Self {
            connection: RwLock::new(CacheConnection {
                url: config.machine_url.clone(),
                token: config.machine_token.clone(),
            }),
            config,
            network,
        }
    }

    async fn dispatch(&self, request: Request<Body>) -> Result<Response<Body>> {
        let (parts, body) = request.into_parts();
        let path = parts
            .uri
            .path_and_query()
            .map_or(parts.uri.path(), |value| value.as_str());
        let registration = &self.config.registration;
        let cache = cache_path(parts.uri.path());
        if cache
            && (save_path(parts.uri.path(), &parts.method) && !registration.permissions.write
                || !save_path(parts.uri.path(), &parts.method) && !registration.permissions.read)
        {
            return Ok(miss(parts.uri.path()));
        }
        let mut headers = parts.headers;
        headers.remove("x-ciproxy-session");
        if !cache && !parts.uri.path().starts_with("/ciproxy/blob/") {
            return self.github(parts.method, path, headers, body).await;
        }
        let connection = self.connection.read().await.clone();
        headers.insert(
            "x-ciproxy-session",
            connection.token.expose_secret().parse().internal()?,
        );
        let machine = format!(
            "{}{}",
            connection.url.expose_secret().trim_end_matches('/'),
            path
        );
        let response = self
            .network
            .send(Outbound {
                method: parts.method,
                url: machine,
                headers,
                body,
            })
            .await?;
        if response.status().is_server_error() || response.status() == StatusCode::UNAUTHORIZED {
            tracing::warn!("Machine cache unavailable; returning a miss");
            return Ok(miss(parts.uri.path()));
        }
        Ok(response)
    }

    async fn github(
        &self,
        method: Method,
        path: &str,
        mut headers: HeaderMap,
        body: Body,
    ) -> Result<Response<Body>> {
        headers.remove("x-ciproxy-session");
        headers.remove("authorization");
        headers.insert(
            "authorization",
            format!(
                "Bearer {}",
                self.config.registration.github_token.expose_secret()
            )
            .parse()
            .internal()?,
        );
        self.network
            .send(Outbound {
                method,
                url: destination(&self.config.registration, path)?,
                headers,
                body,
            })
            .await
    }
}

#[async_trait]
impl ProxyControl for VmProxy {
    async fn replace_connection(&self, connection: CacheConnection) {
        *self.connection.write().await = connection;
    }
}

#[async_trait]
impl Service for VmProxy {
    async fn handle(&self, request: Request<Body>) -> Response<Body> {
        let request = match canonicalize(request) {
            Ok(request) => request,
            Err(_) => return status(StatusCode::NO_CONTENT),
        };
        let path = request.uri().path().to_owned();
        match self.dispatch(request).await {
            Ok(response) => response,
            Err(_) if cache_path(&path) || path.starts_with("/ciproxy/blob/") => {
                tracing::warn!("Machine cache unavailable; returning a miss");
                miss(&path)
            }
            Err(_) => status(StatusCode::BAD_GATEWAY),
        }
    }

    async fn admin(&self, _request: Request<Body>) -> Response<Body> {
        status(StatusCode::NOT_FOUND)
    }
}
