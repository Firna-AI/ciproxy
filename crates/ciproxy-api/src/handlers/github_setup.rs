//! Public GitHub setup return sends pending setup to the terms page.

use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect, Response};

use crate::error::Result;
use crate::state::ApiState;
use crate::wire::access::SetupCallback;

use super::auth::set_oauth_cookie;

#[utoipa::path(operation_id="github_setup_begin",get,path="/github/setup",params(SetupCallback),responses((status=303,description="Dashboard terms page with pending setup, or failed setup redirect")))]
pub(crate) async fn begin(
    State(state): State<ApiState>,
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
    let pending = state
        .access
        .hold_setup(installation, parameters.state, request_only)
        .await?;
    let mut response = Redirect::to("/continue").into_response();
    set_oauth_cookie(response.headers_mut(), &pending)?;
    Ok(response)
}
