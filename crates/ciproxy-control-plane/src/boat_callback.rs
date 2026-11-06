//! Sandbox-scoped runner-exit reports mounted outside browser authentication.

use std::sync::Arc;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
use axum::routing::post;
use axum::{Json, Router};
use ciproxy_overflow::api::Overflow;
use ciproxy_overflow::error::Error;
use ciproxy_protocol::guest::GuestToAgent;
use ciproxy_protocol::secret::Secret;
use uuid::Uuid;

/// Mount only the narrow token-authenticated exit callback.
pub fn router(overflow: Arc<dyn Overflow>) -> Router {
    Router::new()
        .route("/hooks/boat/runner-exit", post(report))
        .with_state(overflow)
}

async fn report(
    State(overflow): State<Arc<dyn Overflow>>,
    headers: HeaderMap,
    Json(message): Json<GuestToAgent>,
) -> StatusCode {
    let Some(token) = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|value| !value.is_empty())
    else {
        return StatusCode::UNAUTHORIZED;
    };
    let GuestToAgent::RunnerExited { runner_id, .. } = message else {
        return StatusCode::BAD_REQUEST;
    };
    let Ok(runner) = runner_id.0.parse::<Uuid>() else {
        return StatusCode::BAD_REQUEST;
    };
    match overflow.report_end(Secret::new(token), runner).await {
        Ok(()) => StatusCode::NO_CONTENT,
        Err(Error::InvalidToken) => StatusCode::UNAUTHORIZED,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}
