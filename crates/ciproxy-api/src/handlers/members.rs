//! Admin-only roles and unanswered access requests.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::{Extension, Json};

use crate::access::Principal;
use crate::error::{Error, Result};
use crate::state::{ApiState, query};
use crate::wire::access::{MemberPath, MembersView, RoleChange, Success, WorkspacePath};

use super::views;

#[utoipa::path(operation_id = "members_list", get, path = "/workspaces/{workspace_id}/members", params(WorkspacePath), responses((status = 200, body = MembersView)), security(("session" = [])))]
pub(crate) async fn list(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<WorkspacePath>,
) -> Result<Json<MembersView>> {
    state
        .access
        .require_role(&principal, path.workspace_id, true)
        .await?;
    let members = query(state.store, move |store| {
        store.dashboard_members(path.workspace_id)
    })
    .await?;
    Ok(Json(MembersView { open_requests: members.iter().filter(|member| member.role.is_none()).count().try_into().unwrap_or(u32::MAX),
        members: members.into_iter().map(views::member).collect(),
        admin_notice: "An Admin can add machines and connect Boat. These run jobs and their secrets for every organization in this workspace.".to_owned() }))
}

#[utoipa::path(operation_id = "members_save", put, path = "/workspaces/{workspace_id}/members/{member_id}", params(MemberPath), request_body = RoleChange, responses((status = 200, body = Success)), security(("session" = [])))]
pub(crate) async fn save(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<MemberPath>,
    Json(body): Json<RoleChange>,
) -> Result<Json<Success>> {
    state
        .access
        .require_role(&principal, path.workspace_id, true)
        .await?;
    let github_id = match path.member_id.parse::<i64>() {
        Ok(value) => value,
        Err(_) => return Err(Error::InvalidInput),
    };
    let at = state.runtime.now();
    query(state.store, move |store| {
        store.save_dashboard_role(
            path.workspace_id,
            github_id,
            Some(views::stored_role(body.role)),
            at,
        )
    })
    .await?;
    Ok(Json(Success { ok: true }))
}

#[utoipa::path(operation_id = "members_remove", delete, path = "/workspaces/{workspace_id}/members/{member_id}", params(MemberPath), responses((status = 200, body = Success)), security(("session" = [])))]
pub(crate) async fn remove(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<MemberPath>,
) -> Result<Json<Success>> {
    state
        .access
        .require_role(&principal, path.workspace_id, true)
        .await?;
    let github_id = match path.member_id.parse::<i64>() {
        Ok(value) => value,
        Err(_) => return Err(Error::InvalidInput),
    };
    let at = state.runtime.now();
    query(state.store, move |store| {
        store.save_dashboard_role(path.workspace_id, github_id, None, at)
    })
    .await?;
    Ok(Json(Success { ok: true }))
}
