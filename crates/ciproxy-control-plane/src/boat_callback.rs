//! Sandbox-scoped exit and usage reports mounted outside browser authentication.

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

/// Mount only the narrow token-authenticated sandbox callback.
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
    let token = Secret::new(token);
    match message {
        GuestToAgent::RunnerExited {
            runner_id, usage, ..
        } => {
            let Ok(runner) = runner_id.0.parse::<Uuid>() else {
                return StatusCode::BAD_REQUEST;
            };
            match overflow.report_end(token, runner, usage).await {
                Ok(()) => StatusCode::NO_CONTENT,
                Err(Error::InvalidToken) => StatusCode::UNAUTHORIZED,
                Err(_) => StatusCode::SERVICE_UNAVAILABLE,
            }
        }
        GuestToAgent::UsageSamples { runner_id, usage } => {
            let Ok(runner) = runner_id.0.parse::<Uuid>() else {
                return StatusCode::BAD_REQUEST;
            };
            match overflow.report_samples(token, runner, usage).await {
                Ok(()) => StatusCode::NO_CONTENT,
                Err(Error::InvalidToken) => StatusCode::UNAUTHORIZED,
                Err(Error::SamplesClosed) => StatusCode::GONE,
                Err(Error::SamplesTooSoon) => StatusCode::TOO_MANY_REQUESTS,
                Err(Error::InvalidSamples) => StatusCode::UNPROCESSABLE_ENTITY,
                Err(_) => {
                    tracing::warn!("dropped Boat sample report: store operation failed");
                    StatusCode::SERVICE_UNAVAILABLE
                }
            }
        }
        _ => StatusCode::BAD_REQUEST,
    }
}
