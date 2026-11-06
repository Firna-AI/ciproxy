//! Public GitHub setup return initiates fresh user association proof.

use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Redirect, Response};
use secrecy::ExposeSecret;

use crate::error::Result;
use crate::router::cookie;
use crate::state::ApiState;
use crate::wire::access::SetupCallback;

use super::auth::set_cookie;

#[utoipa::path(operation_id="github_setup_begin",get,path="/github/setup",params(SetupCallback),responses((status=303,description="Fresh GitHub authorization or failed setup redirect")))]
pub(crate) async fn begin(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Query(parameters): Query<SetupCallback>,
) -> Result<Response> {
    let request_only = parameters.setup_action.as_deref() == Some("request");
    let installation = if request_only {
        None
    } else {
        let Some(id) = parameters
            .installation_id
            .and_then(|id| id.parse::<i64>().ok())
        else {
            return Ok(Redirect::to("/sign-in?reason=github").into_response());
        };
        Some(match state.installations.installation(id).await {
            Ok(Some(installation)) => installation,
            _ => return Ok(Redirect::to("/sign-in?reason=github").into_response()),
        })
    };
    let previous = cookie(&headers, "__Host-ciproxy");
    let attempt = state
        .access
        .begin_setup(
            installation,
            parameters.state,
            request_only,
            previous.as_deref(),
        )
        .await?;
    let mut response = Redirect::to(&attempt.url).into_response();
    set_cookie(
        response.headers_mut(),
        "__Host-ciproxy-oauth",
        attempt.binding.expose_secret(),
        600,
        true,
    )?;
    Ok(response)
}
