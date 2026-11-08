//! Mountable axum router with complete-response build and browser protections.

use std::sync::Arc;

use axum::extract::{MatchedPath, Request, State};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::middleware::{Next, from_fn_with_state};
use axum::response::{IntoResponse, Response};
use axum::{Json, Router};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::error::{Error, Problem, ProblemCode};
use crate::handlers::{
    auth, billing_pages, checkout, github_setup, jobs, labels, links, machines, members,
    organizations, overflow, savings, sessions, setup,
};
use crate::state::ApiState;

pub(crate) fn routes() -> OpenApiRouter<ApiState> {
    OpenApiRouter::new()
        .routes(routes!(auth::begin))
        .routes(routes!(auth::callback))
        .routes(routes!(auth::sign_out))
        .routes(routes!(github_setup::begin))
        .routes(routes!(sessions::session))
        .routes(routes!(sessions::claim))
        .routes(routes!(members::list))
        .routes(routes!(members::save))
        .routes(routes!(members::remove))
        .routes(routes!(machines::list))
        .routes(routes!(machines::detail))
        .routes(routes!(machines::warm_runners))
        .routes(routes!(machines::cost))
        .routes(routes!(machines::command))
        .routes(routes!(machines::issue_token))
        .routes(routes!(machines::revoke_token))
        .routes(routes!(setup::setup))
        .routes(routes!(setup::settings))
        .routes(routes!(setup::rename))
        .routes(routes!(organizations::list))
        .routes(routes!(organizations::repositories))
        .routes(routes!(organizations::save_repositories))
        .routes(routes!(organizations::takeout))
        .routes(routes!(labels::list))
        .routes(routes!(labels::preview))
        .routes(routes!(labels::create))
        .routes(routes!(labels::update))
        .routes(routes!(labels::remove))
        .routes(routes!(links::begin))
        .routes(routes!(links::start))
        .routes(routes!(links::list))
        .routes(routes!(links::detail))
        .routes(routes!(links::choose))
        .routes(routes!(links::enable))
        .routes(routes!(overflow::settings))
        .routes(routes!(overflow::summary))
        .routes(routes!(overflow::wallets))
        .routes(routes!(overflow::connect))
        .routes(routes!(overflow::disconnect))
        .routes(routes!(overflow::save))
        .routes(routes!(jobs::live))
        .routes(routes!(jobs::history))
        .routes(routes!(jobs::detail))
        .routes(routes!(savings::report))
        .routes(routes!(billing_pages::usage))
        .routes(routes!(billing_pages::plans))
        .routes(routes!(billing_pages::banner))
        .routes(routes!(checkout::review))
        .routes(routes!(checkout::open))
        .routes(routes!(checkout::returned))
        .routes(routes!(checkout::change))
        .routes(routes!(checkout::addons))
        .routes(routes!(checkout::free))
        .routes(routes!(checkout::portal))
}

/// Build the complete dashboard router. Mount it under /api at the host root.
pub fn router(state: ApiState) -> Router {
    let (router, _) = routes().split_for_parts();
    router
        .fallback(|| async { Error::NotFound })
        .layer(from_fn_with_state(state.clone(), protect))
        .with_state(state)
}

pub(crate) fn cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|header| header.to_str().ok())
        .flat_map(|header| header.split(';'))
        .filter_map(|piece| piece.trim().split_once('='))
        .find_map(|(key, value)| (key == name).then(|| value.to_owned()))
}

pub(crate) async fn protect(
    State(state): State<ApiState>,
    mut request: Request,
    next: Next,
) -> Response {
    let public = matches!(
        request.uri().path(),
        "/auth/github" | "/auth/github/callback" | "/github/setup" | "/hooks/stripe"
    ) || request.extensions().get::<MatchedPath>().is_none();
    let result = async {
        if !public {
            let token = cookie(request.headers(), "__Host-ciproxy").ok_or(Error::Unauthorized)?;
            let principal = state.access.authenticate(&token).await?;
            if !matches!(
                *request.method(),
                Method::GET | Method::HEAD | Method::OPTIONS
            ) {
                let origin = request
                    .headers()
                    .get(header::ORIGIN)
                    .and_then(|value| value.to_str().ok());
                let csrf = request
                    .headers()
                    .get("x-ciproxy-csrf")
                    .and_then(|value| value.to_str().ok());
                if origin != Some(state.config.public_origin.as_str())
                    || !csrf.is_some_and(|value| principal.accepts_csrf(value))
                {
                    return Err(Error::Forbidden);
                }
            }
            request.extensions_mut().insert(Arc::new(principal));
        }
        Ok(next.run(request).await)
    }
    .await;
    let mut response = match result {
        Ok(response) => response,
        Err(error) => error.into_response(),
    };
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    if response.status().is_client_error() && !content_type.starts_with("application/json") {
        let status = response.status();
        let code = if status == StatusCode::METHOD_NOT_ALLOWED {
            ProblemCode::MethodNotAllowed
        } else {
            ProblemCode::InvalidRequest
        };
        response = (
            status,
            Json(Problem {
                code,
                message: "The request could not be processed.".to_owned(),
            }),
        )
            .into_response();
    }
    let build =
        HeaderValue::from_str(&state.config.build).unwrap_or(HeaderValue::from_static("unknown"));
    response.headers_mut().insert("x-ciproxy-build", build);
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}
