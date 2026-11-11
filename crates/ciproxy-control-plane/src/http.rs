//! Shared dashboard, agent, and authenticated GitHub HTTP routes.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use axum::Router;
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use ciproxy_api::pages::{PageServices, router_with_pages, stripe_router};
use ciproxy_api::state::ApiState;
use ciproxy_github::error::Error;
use ciproxy_github::hooks::WebhookHandler;

/// Mount every dashboard page and its raw Stripe hook on the shared host.
pub fn dashboard_router(
    agent: Router,
    state: ApiState,
    pages: Arc<PageServices>,
    hooks: Arc<dyn WebhookHandler>,
    ready: Arc<AtomicBool>,
) -> Router {
    let stripe = stripe_router(state.clone(), pages.billing.clone());
    router(agent, router_with_pages(state, pages), hooks, ready).merge(stripe)
}

/// Mount an API router beside the agent transport and GitHub webhooks.
pub fn router(
    agent: Router,
    api: Router,
    hooks: Arc<dyn WebhookHandler>,
    ready: Arc<AtomicBool>,
) -> Router {
    let hooks = Router::new()
        .route("/hooks/github", post(webhook))
        .layer(DefaultBodyLimit::max(25 * 1024 * 1024))
        .with_state(hooks);
    Router::new()
        .merge(agent)
        .merge(hooks)
        .nest("/api", api)
        .route(
            "/api/health/ready",
            get(move || {
                let ready = ready.clone();
                async move {
                    if ready.load(Ordering::Acquire) {
                        StatusCode::OK
                    } else {
                        StatusCode::SERVICE_UNAVAILABLE
                    }
                }
            }),
        )
        .route("/api/health/live", get(|| async { StatusCode::OK }))
}

async fn webhook(
    State(handler): State<Arc<dyn WebhookHandler>>,
    headers: HeaderMap,
    bytes: Bytes,
) -> StatusCode {
    let Some(event) = headers
        .get("x-github-event")
        .and_then(|value| value.to_str().ok())
    else {
        return StatusCode::BAD_REQUEST;
    };
    let Some(signature) = headers
        .get("x-hub-signature-256")
        .and_then(|value| value.to_str().ok())
    else {
        return StatusCode::UNAUTHORIZED;
    };
    match handler.handle(event, signature, &bytes).await {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(Error::InvalidSignature) => StatusCode::UNAUTHORIZED,
        Err(error) => {
            tracing::warn!(%error, "GitHub webhook failed; delivery-history recovery will retry");
            StatusCode::SERVICE_UNAVAILABLE
        }
    }
}

#[cfg(test)]
#[path = "_tests_/http_tests.rs"]
mod http_tests;
