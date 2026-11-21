//! Safe HTTP errors with a generated public error payload.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use internal_error::{ErrorContract, InternalError};
use serde::Serialize;
use utoipa::ToSchema;

/// Stable error body returned by every dashboard route.
#[derive(Serialize, ToSchema)]
pub struct Problem {
    /// Stable machine-readable error identifier.
    pub code: ProblemCode,
    /// Safe explanation with no credentials or provider response body.
    pub message: String,
}

/// Closed set of machine-readable dashboard failures.
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProblemCode {
    /// Boat refused the submitted API key.
    BoatKeyRejected,
    /// GitHub sign-in is absent, expired, or revoked.
    SignInRequired,
    /// Workspace access or browser proof did not authorize this request.
    AccessDenied,
    /// The resource does not exist in the authorized workspace.
    NotFound,
    /// Saved state conflicts with the operation.
    Conflict,
    /// The paid quote cannot start another payment.
    ReviewRequired,
    /// The request cannot represent this operation.
    InvalidRequest,
    /// GitHub refused the requested label name.
    GithubLabelRefused,
    /// The upstream service or agent is unavailable.
    Unavailable,
    /// An unhandled internal operation failed.
    Internal,
    /// The route has no operation for this HTTP method.
    MethodNotAllowed,
}

/// Failures that dashboard callers can handle without inspecting diagnostics.
#[derive(Debug, thiserror::Error, ErrorContract)]
pub enum Error {
    /// Boat refuses the submitted API key without exposing it.
    #[error("[api] Boat rejected the key")]
    BoatKeyRejected,
    /// Authentication is absent, expired, or revoked.
    #[error("[api] sign in with GitHub again")]
    Unauthorized,
    /// Workspace access or browser proof did not authorize this request.
    #[error("[api] workspace access denied")]
    Forbidden,
    /// No resource exists in the authorized workspace.
    #[error("[api] resource not found")]
    NotFound,
    /// The operation conflicts with the saved workspace state.
    #[error("[api] operation conflicts with the workspace state")]
    Conflict,
    /// The payer must review a new quote before starting payment.
    #[error("[api] review the paid change again")]
    ReviewRequired,
    /// The request cannot represent the documented resource.
    #[error("[api] request is invalid")]
    InvalidInput,
    /// GitHub refuses a label name.
    #[error("[api] GitHub refused this label name")]
    LabelRefused {
        /// GitHub validation text from a refused scale-set creation.
        message: String,
    },
    /// GitHub or an agent cannot currently complete the operation.
    #[error("[api] service is temporarily unavailable")]
    Unavailable,
    /// An internal failure whose source never enters an HTTP response.
    #[error("[api/internal] request failed")]
    Internal(#[from] InternalError),
}

/// Result used by HTTP services and adapters.
pub type Result<T> = std::result::Result<T, Error>;

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            Self::BoatKeyRejected => (
                StatusCode::UNPROCESSABLE_ENTITY,
                ProblemCode::BoatKeyRejected,
            ),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, ProblemCode::SignInRequired),
            Self::Forbidden => (StatusCode::FORBIDDEN, ProblemCode::AccessDenied),
            Self::NotFound => (StatusCode::NOT_FOUND, ProblemCode::NotFound),
            Self::Conflict => (StatusCode::CONFLICT, ProblemCode::Conflict),
            Self::ReviewRequired => (StatusCode::CONFLICT, ProblemCode::ReviewRequired),
            Self::InvalidInput => (StatusCode::BAD_REQUEST, ProblemCode::InvalidRequest),
            Self::LabelRefused { .. } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                ProblemCode::GithubLabelRefused,
            ),
            Self::Unavailable => (StatusCode::SERVICE_UNAVAILABLE, ProblemCode::Unavailable),
            Self::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, ProblemCode::Internal),
        };
        let message = match self {
            Self::BoatKeyRejected => {
                "Boat did not accept this key. Copy the whole key from the Boat console.".to_owned()
            }
            Self::Unauthorized => "Sign in with GitHub again.".to_owned(),
            Self::Forbidden => "This request is not permitted.".to_owned(),
            Self::NotFound => "This resource was not found.".to_owned(),
            Self::Conflict => "This action conflicts with the current workspace state.".to_owned(),
            Self::ReviewRequired => "Review this paid change again before you pay.".to_owned(),
            Self::InvalidInput => "Check the request and try again.".to_owned(),
            Self::LabelRefused { message } => message,
            Self::Unavailable => "The action could not finish. Try again.".to_owned(),
            Self::Internal(_) => "The request could not finish. Try again.".to_owned(),
        };
        (status, Json(Problem { code, message })).into_response()
    }
}

/// Preserve handled service failures at the HTTP boundary.
pub(crate) trait ServiceResultExt<T> {
    /// Keep authorization, absence, and availability distinct from internal faults.
    #[track_caller]
    fn api(self) -> Result<T>;
}

impl<T> ServiceResultExt<T> for ciproxy_github::error::Result<T> {
    #[track_caller]
    fn api(self) -> Result<T> {
        match self {
            Ok(value) => Ok(value),
            Err(
                ciproxy_github::error::Error::AccessUnavailable(_)
                | ciproxy_github::error::Error::InstallationPending,
            ) => Err(Error::Unavailable),
            Err(
                ciproxy_github::error::Error::WorkspaceConflict
                | ciproxy_github::error::Error::OrganizationInactive
                | ciproxy_github::error::Error::RunnerBusy,
            ) => Err(Error::Conflict),
            Err(source) => Err(Error::internal(source)),
        }
    }
}
impl<T> ServiceResultExt<T> for ciproxy_gateway::error::Result<T> {
    #[track_caller]
    fn api(self) -> Result<T> {
        match self {
            Ok(value) => Ok(value),
            Err(ciproxy_gateway::error::Error::Forbidden) => Err(Error::Forbidden),
            Err(ciproxy_gateway::error::Error::Unauthorized) => Err(Error::Unauthorized),
            Err(ciproxy_gateway::error::Error::NotFound) => Err(Error::NotFound),
            Err(source) => Err(Error::internal(source)),
        }
    }
}
