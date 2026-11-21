//! HTTP metadata route without job-controlled repository or ref authority.

use std::sync::Arc;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use ciproxy_protocol::cache_metadata::{
    CACHE_METADATA_PATH, CacheMetadataRequest, CacheMetadataResponse,
};
use ciproxy_protocol::secret::Secret;

use crate::error::Error;

use super::service::CacheMetadata;

/// Mount the host-only assignment metadata operation on the shared gateway host.
pub(crate) fn router(service: Arc<dyn CacheMetadata>) -> Router {
    Router::new()
        .route(CACHE_METADATA_PATH, post(lookup))
        .with_state(service)
}

async fn lookup(
    State(service): State<Arc<dyn CacheMetadata>>,
    headers: HeaderMap,
    Json(request): Json<CacheMetadataRequest>,
) -> Result<Json<CacheMetadataResponse>, StatusCode> {
    let token = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;
    match service.lookup(Secret::new(token), request).await {
        Ok(response) => Ok(Json(response)),
        Err(Error::RunnerOwnership) => Err(StatusCode::UNAUTHORIZED),
        Err(_) => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}
