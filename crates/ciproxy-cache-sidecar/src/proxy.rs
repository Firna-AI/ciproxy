//! VM proxy with a trusted fallback policy and streaming results passthrough.

use std::sync::Arc;

use async_trait::async_trait;
use axum::body::{Body, to_bytes};
use axum::http::{HeaderMap, Method, Request, Response, StatusCode};
use ciproxy_protocol::secret::Secret;
use serde::{Deserialize, Serialize};

use crate::error::{Result, ResultExt};
use crate::scope::can_save;
use crate::service::Service;
use crate::session::Registration;
use crate::transport::{Outbound, Transport};
use crate::upstream::{canonicalize, destination};
use crate::wire::{cache_path, miss, save_path, status};

/// Agent-written configuration for exactly one disposable VM proxy.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProxyConfig {
    /// Narrow machine-cache listener URL, reachable from this VM.
    pub machine_url: Secret,
    /// Credential for this VM's registered session only.
    pub machine_token: Secret,
    /// Original assignment facts, permissions, and GitHub endpoints.
    pub registration: Registration,
}

/// VM HTTP forwarding boundary. Consume it through `dyn Service`.
pub struct VmProxy {
    config: ProxyConfig,
    network: Arc<dyn Transport>,
}

impl VmProxy {
    /// Compose a proxy from trusted assignment data and injected HTTP IO.
    pub fn new(config: ProxyConfig, network: Arc<dyn Transport>) -> Self {
        Self { config, network }
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
            && (save_path(parts.uri.path(), &parts.method)
                && !can_save(&registration.facts, registration.permissions.write)
                || !save_path(parts.uri.path(), &parts.method) && !registration.permissions.read)
        {
            return Ok(miss(parts.uri.path()));
        }
        let mut headers = parts.headers;
        headers.remove("x-ciproxy-session");
        if !cache && !parts.uri.path().starts_with("/ciproxy/blob/") {
            return self.github(parts.method, path, headers, body).await;
        }
        headers.insert(
            "x-ciproxy-session",
            self.config
                .machine_token
                .expose_secret()
                .parse()
                .internal()?,
        );
        let machine = format!(
            "{}{}",
            self.config
                .machine_url
                .expose_secret()
                .trim_end_matches('/'),
            path
        );
        if cache && parts.method != Method::PATCH {
            let bytes = to_bytes(body, 1024 * 1024).await.internal()?;
            let response = self
                .network
                .send(Outbound {
                    method: parts.method.clone(),
                    url: machine,
                    headers: headers.clone(),
                    body: Body::from(bytes.clone()),
                })
                .await;
            match response {
                Ok(response)
                    if response.status().is_success()
                        || response.status().is_client_error()
                            && response.status() != StatusCode::UNAUTHORIZED =>
                {
                    return Ok(response);
                }
                _ => tracing::warn!("Machine cache unavailable; using GitHub"),
            }
            self.report_fallback().await;
            let response = self
                .github(parts.method, path, headers, Body::from(bytes))
                .await?;
            return Ok(if response.status().is_server_error() {
                miss(parts.uri.path())
            } else {
                response
            });
        }
        self.network
            .send(Outbound {
                method: parts.method,
                url: machine,
                headers,
                body,
            })
            .await
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

    async fn report_fallback(&self) {
        let mut headers = HeaderMap::new();
        if let Ok(token) = self.config.machine_token.expose_secret().parse() {
            headers.insert("x-ciproxy-session", token);
            let _ = self
                .network
                .send(Outbound {
                    method: Method::POST,
                    url: format!(
                        "{}/ciproxy/fallback",
                        self.config
                            .machine_url
                            .expose_secret()
                            .trim_end_matches('/')
                    ),
                    headers,
                    body: Body::empty(),
                })
                .await;
        }
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
                self.report_fallback().await;
                miss(&path)
            }
            Err(_) => status(StatusCode::BAD_GATEWAY),
        }
    }

    async fn admin(&self, _request: Request<Body>) -> Response<Body> {
        status(StatusCode::NOT_FOUND)
    }
}
