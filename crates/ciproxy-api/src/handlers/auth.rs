//! Browser-bound OAuth redirects and current-session sign-out.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::{HeaderMap, HeaderValue, header};
use axum::response::{IntoResponse, Redirect, Response};
use axum::{Extension, Json};
use secrecy::ExposeSecret;

use crate::access::Principal;
use crate::error::{ErrorResultExt, Result};
use crate::router::cookie;
use crate::state::ApiState;
use crate::wire::access::{OAuthCallback, Success};

pub(crate) fn set_cookie(
    headers: &mut HeaderMap,
    name: &str,
    value: &str,
    age: i64,
    http_only: bool,
) -> Result<()> {
    let value = format!(
        "{name}={value}; Path=/; Secure; SameSite=Lax; Max-Age={age}{}",
        if http_only { "; HttpOnly" } else { "" }
    );
    headers.append(
        header::SET_COOKIE,
        HeaderValue::from_str(&value).internal()?,
    );
    Ok(())
}

#[utoipa::path(operation_id = "auth_begin", get, path = "/auth/github", responses((status = 303, description = "GitHub App authorization redirect")))]
pub(crate) async fn begin(State(state): State<ApiState>, headers: HeaderMap) -> Result<Response> {
    let previous = cookie(&headers, "__Host-ciproxy");
    let attempt = state.access.begin_login(previous.as_deref()).await?;
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

#[utoipa::path(operation_id = "auth_callback", get, path = "/auth/github/callback", params(OAuthCallback), responses((status = 303, description = "Dashboard or failed sign-in redirect")))]
pub(crate) async fn callback(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Query(parameters): Query<OAuthCallback>,
) -> Result<Response> {
    if parameters.error.is_some() {
        return Ok(Redirect::to("/sign-in?reason=github").into_response());
    }
    let (Some(code), Some(nonce), Some(binding)) = (
        parameters.code,
        parameters.state,
        cookie(&headers, "__Host-ciproxy-oauth"),
    ) else {
        return Ok(Redirect::to("/sign-in?reason=github").into_response());
    };
    let result = match state
        .access
        .finish_login(code.into(), &nonce, &binding)
        .await
    {
        Ok(result) => result,
        Err(_) => return Ok(Redirect::to("/sign-in?reason=github").into_response()),
    };
    let mut response = Redirect::to("/").into_response();
    set_cookie(response.headers_mut(), "__Host-ciproxy-oauth", "", 0, true)?;
    let age = (result.expires_at - state.runtime.now()).num_seconds();
    set_cookie(
        response.headers_mut(),
        "__Host-ciproxy",
        result.cookie.expose_secret(),
        age,
        true,
    )?;
    set_cookie(
        response.headers_mut(),
        "__Host-ciproxy-csrf",
        result.csrf.expose_secret(),
        age,
        false,
    )?;
    Ok(response)
}

#[utoipa::path(operation_id = "auth_sign_out", post, path = "/auth/sign-out", responses((status = 200, body = Success)), security(("session" = [])))]
pub(crate) async fn sign_out(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
) -> Result<Response> {
    state.access.sign_out(&principal).await?;
    let mut response = Json(Success { ok: true }).into_response();
    set_cookie(response.headers_mut(), "__Host-ciproxy", "", 0, true)?;
    set_cookie(response.headers_mut(), "__Host-ciproxy-csrf", "", 0, false)?;
    Ok(response)
}
