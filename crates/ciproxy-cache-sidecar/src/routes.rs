//! Agent-only session, clear, capacity, and report operations.

use std::collections::HashMap;
use std::sync::Arc;

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, Response, StatusCode};
use ciproxy_protocol::cache_metadata::{CacheMetadataAnswer, CacheMetadataRelayConfig};
use ciproxy_protocol::secret::Secret;
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::error::{Result, ResultExt};
use crate::machine::{MachineCache, Session, identifier};
use crate::session::SessionCredential;
use crate::wire::{json, status};
use ciproxy_protocol::runtime::CacheRegistration;

#[derive(Deserialize)]
struct Budget {
    capacity_bytes: u64,
}

impl MachineCache {
    pub(crate) async fn administration(&self, request: Request<Body>) -> Result<Response<Body>> {
        let (parts, body) = request.into_parts();
        let path = parts.uri.path();
        match (parts.method, path) {
            (Method::POST, "/sessions/recover") => {
                let bytes = to_bytes(body, 1024 * 1024).await.internal()?;
                let registration: CacheRegistration = serde_json::from_slice(&bytes).internal()?;
                let credential = self
                    .sessions
                    .lock()
                    .await
                    .iter()
                    .find(|(_, session)| session.registration == registration)
                    .map(|(token, session)| SessionCredential {
                        token: Secret::new(token),
                        scope: session.scope.clone(),
                    });
                Ok(json(&credential))
            }
            (Method::POST, path) if path == "/sessions/close" || path == "/sessions/fallback" => {
                let bytes = to_bytes(body, 1024 * 1024).await.internal()?;
                let facts: ciproxy_protocol::jobs::JobFacts =
                    serde_json::from_slice(&bytes).internal()?;
                let mut sessions = self.sessions.lock().await;
                let tokens: Vec<_> = sessions
                    .iter()
                    .filter(|(_, session)| {
                        session.registration.facts == facts
                            && facts.assignment.job_id.as_ref()
                                == Some(&session.registration.job_id)
                    })
                    .map(|(token, _)| token.clone())
                    .collect();
                for token in tokens {
                    if path == "/sessions/fallback" {
                        if let Some(session) = sessions.get(&token) {
                            self.metrics.fallback(&session.registration).await;
                        }
                    } else if let Some(session) = sessions.remove(&token) {
                        self.metrics.closed(&session.registration).await;
                    }
                }
                Ok(status(StatusCode::NO_CONTENT))
            }
            (Method::POST, "/sessions") => {
                let bytes = to_bytes(body, 1024 * 1024).await.internal()?;
                let registration: CacheRegistration = serde_json::from_slice(&bytes).internal()?;
                let scope = self
                    .tokens
                    .check(&registration.github_token, &registration.facts)
                    .await;
                self.metrics.registered(&registration, &scope).await;
                let token = identifier();
                self.sessions.lock().await.insert(
                    token.clone(),
                    Arc::new(Session {
                        registration,
                        scope: scope.clone(),
                        private_ref: format!("private-session/{}", identifier()),
                        uploads: Mutex::new(HashMap::new()),
                        downloads: Mutex::new(HashMap::new()),
                    }),
                );
                Ok(json(&SessionCredential {
                    token: Secret::new(token),
                    scope,
                }))
            }
            (Method::DELETE, path) if path.starts_with("/sessions/") => {
                if let Some(session) = self
                    .sessions
                    .lock()
                    .await
                    .remove(&path["/sessions/".len()..])
                {
                    self.metrics.closed(&session.registration).await;
                }
                Ok(status(StatusCode::NO_CONTENT))
            }
            (Method::GET, "/report") => Ok(json(&self.metrics.report().await?)),
            (Method::POST, "/clear") => {
                self.store.clear().await?;
                Ok(status(StatusCode::NO_CONTENT))
            }
            (Method::PUT, "/metadata-relay") => {
                let bytes = to_bytes(body, 1024).await.internal()?;
                let config: CacheMetadataRelayConfig = serde_json::from_slice(&bytes).internal()?;
                self.metadata_requests.enable(config.enabled).await;
                Ok(status(StatusCode::NO_CONTENT))
            }
            (Method::GET, "/metadata-requests") => Ok(json(&self.metadata_requests.next().await)),
            (Method::POST, "/metadata-responses") => {
                let bytes = to_bytes(body, 16 * 1024 * 1024).await.internal()?;
                let answer: CacheMetadataAnswer = serde_json::from_slice(&bytes).internal()?;
                self.metadata_requests.answer(answer).await;
                Ok(status(StatusCode::NO_CONTENT))
            }
            (Method::PUT, "/budget") => {
                let bytes = to_bytes(body, 1024).await.internal()?;
                let budget: Budget = serde_json::from_slice(&bytes).internal()?;
                self.store.resize(budget.capacity_bytes).await?;
                Ok(status(StatusCode::NO_CONTENT))
            }
            _ => Ok(status(StatusCode::NOT_FOUND)),
        }
    }
}
