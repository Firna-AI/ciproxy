//! Admin-only Boat account verification, connection, and routing settings.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::{Extension, Json};
use ciproxy_overflow::api::Overflow;
use ciproxy_store_interface::settings::{OverflowWait, Settings};

use crate::access::Principal;
use crate::error::{Error, Result};
use crate::state::{ApiState, query};
use crate::wire::access::{Success, WorkspacePath};
use crate::wire::overflow::{BoatConnection, OverflowPolicy, OverflowSettings};

use super::overflow_views;

#[utoipa::path(operation_id="overflow_settings", get, path="/workspaces/{workspace_id}/overflow", params(WorkspacePath), responses((status=200,body=OverflowSettings)), security(("session"=[])))]
pub(crate) async fn settings(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<WorkspacePath>,
) -> Result<Json<OverflowSettings>> {
    let workspace = state
        .access
        .require_role(&principal, path.workspace_id, true)
        .await?;
    Ok(Json(overflow_views::read(&state, workspace).await?))
}

#[utoipa::path(operation_id="overflow_summary", get, path="/workspaces/{workspace_id}/overflow/summary", params(WorkspacePath), responses((status=200,body=OverflowSettings)), security(("session"=[])))]
pub(crate) async fn summary(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<WorkspacePath>,
) -> Result<Json<OverflowSettings>> {
    let workspace = state
        .access
        .require_role(&principal, path.workspace_id, false)
        .await?;
    Ok(Json(overflow_views::read(&state, workspace).await?))
}

#[utoipa::path(operation_id="overflow_connect", post, path="/workspaces/{workspace_id}/overflow/connection", params(WorkspacePath), request_body=BoatConnection, responses((status=200,body=Success)), security(("session"=[])))]
pub(crate) async fn connect(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<WorkspacePath>,
    Json(body): Json<BoatConnection>,
) -> Result<Json<Success>> {
    let workspace = state
        .access
        .require_role(&principal, path.workspace_id, true)
        .await?;
    let limit = overflow_views::limit(body.spend_limit_cents)?;
    let result = service(&state)?
        .connect(
            workspace,
            state.runtime.identifier(),
            body.api_key,
            body.wallet,
            limit,
        )
        .await;
    overflow_views::result(result)?;
    Ok(Json(Success { ok: true }))
}

#[utoipa::path(operation_id="overflow_disconnect", delete, path="/workspaces/{workspace_id}/overflow/connection", params(WorkspacePath), responses((status=200,body=Success)), security(("session"=[])))]
pub(crate) async fn disconnect(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<WorkspacePath>,
) -> Result<Json<Success>> {
    let workspace = state
        .access
        .require_role(&principal, path.workspace_id, true)
        .await?;
    overflow_views::result(service(&state)?.disconnect(workspace).await)?;
    Ok(Json(Success { ok: true }))
}

#[utoipa::path(operation_id="overflow_save", put, path="/workspaces/{workspace_id}/overflow", params(WorkspacePath), request_body=OverflowPolicy, responses((status=200,body=Success)), security(("session"=[])))]
pub(crate) async fn save(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<WorkspacePath>,
    Json(body): Json<OverflowPolicy>,
) -> Result<Json<Success>> {
    let workspace = state
        .access
        .require_role(&principal, path.workspace_id, true)
        .await?;
    let overflow_wait = match body.wait_seconds {
        0 => OverflowWait::AtOnce,
        15 => OverflowWait::Seconds15,
        30 => OverflowWait::Seconds30,
        60 => OverflowWait::Seconds60,
        120 => OverflowWait::Seconds120,
        _ => return Err(Error::InvalidInput),
    };
    let settings = Settings {
        overflow_enabled: body.enabled,
        overflow_wait,
        boat_spend_limit_usd_micros: Some(overflow_views::limit(body.spend_limit_cents)?),
    };
    query(state.store, move |store| {
        store.save_settings(workspace, &settings)
    })
    .await?;
    Ok(Json(Success { ok: true }))
}

pub(super) fn service(state: &ApiState) -> Result<&Arc<dyn Overflow>> {
    state.overflow.as_ref().ok_or(Error::Unavailable)
}
