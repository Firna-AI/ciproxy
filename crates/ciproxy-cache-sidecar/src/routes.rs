//! Agent-only session, clear, capacity, and report operations.

use std::collections::HashMap;
use std::sync::Arc;

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, Response, StatusCode};
use ciproxy_protocol::secret::Secret;
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::error::{Result, ResultExt};
use crate::machine::{MachineCache, Session, identifier};
use crate::session::{Registration, SessionCredential};
use crate::wire::{json, status};

#[derive(Deserialize)]
struct Budget {
    capacity_bytes: u64,
}

impl MachineCache {
    pub(crate) async fn administration(&self, request: Request<Body>) -> Result<Response<Body>> {
        let (parts, body) = request.into_parts();
        let path = parts.uri.path();
        match (parts.method, path) {
            (Method::POST, "/sessions") => {
                let bytes = to_bytes(body, 1024 * 1024).await.internal()?;
                let registration: Registration = serde_json::from_slice(&bytes).internal()?;
                self.metrics.registered(&registration.job_id).await;
                let token = identifier();
                self.sessions.lock().await.insert(
                    token.clone(),
                    Arc::new(Session {
                        registration,
                        uploads: Mutex::new(HashMap::new()),
                        downloads: Mutex::new(HashMap::new()),
                    }),
                );
                Ok(json(&SessionCredential {
                    token: Secret::new(token),
                }))
            }
            (Method::DELETE, path) if path.starts_with("/sessions/") => {
                if let Some(session) = self
                    .sessions
                    .lock()
                    .await
                    .remove(&path["/sessions/".len()..])
                {
                    self.metrics.closed(&session.registration.job_id).await;
                }
                Ok(status(StatusCode::NO_CONTENT))
            }
            (Method::GET, "/report") => Ok(json(&self.metrics.report().await?)),
            (Method::POST, "/clear") => {
                self.store.clear().await?;
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
