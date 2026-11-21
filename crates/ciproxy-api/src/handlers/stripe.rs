//! Raw signed Stripe deliveries outside the browser authentication boundary.

use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::middleware::from_fn_with_state;
use axum::{Extension, Json, Router};
use ciproxy_billing::service::api::BillingApi;
use ciproxy_billing::service::view::EventOutcome;
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::error::{Error, ErrorResultExt, Result};
use crate::router::protect;
use crate::state::ApiState;

use super::billing_call;

#[derive(Clone)]
struct StripeBilling(Arc<dyn BillingApi>);

/// The durable result of an authenticated Stripe delivery.
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum StripeReceipt {
    Applied,
    Duplicate,
    Ignored,
}

pub(crate) fn routes() -> OpenApiRouter<ApiState> {
    OpenApiRouter::new().routes(routes!(webhook))
}

pub(crate) fn router(state: ApiState, billing: Arc<dyn BillingApi>) -> Router {
    let (router, _) = routes().split_for_parts();
    router
        .layer(Extension(StripeBilling(billing)))
        .layer(from_fn_with_state(state.clone(), protect))
        .with_state(state)
}

#[utoipa::path(operation_id="stripe_webhook",post,path="/hooks/stripe",request_body(content=Object,content_type="application/json"),params(("Stripe-Signature"=String,Header,description="Stripe signature over the exact raw bytes")),responses((status=200,body=StripeReceipt)))]
async fn webhook(
    State(_state): State<ApiState>,
    Extension(billing): Extension<StripeBilling>,
    headers: HeaderMap,
    raw: Bytes,
) -> Result<Json<StripeReceipt>> {
    let signature = headers
        .get("Stripe-Signature")
        .and_then(|value| value.to_str().ok())
        .ok_or(Error::InvalidInput)?
        .to_owned();
    let outcome = billing_call::result(
        tokio::task::spawn_blocking(move || billing.0.webhook(&signature, &raw))
            .await
            .internal()?,
    )?;
    Ok(Json(match outcome {
        EventOutcome::Applied => StripeReceipt::Applied,
        EventOutcome::Duplicate => StripeReceipt::Duplicate,
        EventOutcome::Ignored => StripeReceipt::Ignored,
    }))
}
