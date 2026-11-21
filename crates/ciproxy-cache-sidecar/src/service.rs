//! Object-safe HTTP service and common streaming router.

use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, Response};
use axum::{Router, extract::State};

/// HTTP boundary for either the machine service or a VM proxy.
#[async_trait]
pub trait Service: Send + Sync {
    /// Handle a request from a job VM.
    async fn handle(&self, request: Request<Body>) -> Response<Body>;
    /// Handle a request from the machine's agent.
    async fn admin(&self, request: Request<Body>) -> Response<Body>;
}

/// Build a job listener that has no administration routes.
pub fn job_router(service: Arc<dyn Service>) -> Router {
    Router::new().fallback(job).with_state(service)
}

/// Build a separate administration listener for the machine agent.
pub fn admin_router(service: Arc<dyn Service>) -> Router {
    Router::new().fallback(admin).with_state(service)
}

async fn job(State(service): State<Arc<dyn Service>>, request: Request<Body>) -> Response<Body> {
    service.handle(request).await
}

async fn admin(State(service): State<Arc<dyn Service>>, request: Request<Body>) -> Response<Body> {
    service.admin(request).await
}
