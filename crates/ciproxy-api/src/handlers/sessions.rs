//! Workspace access, pending requests, and owner recovery through HTTP.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::{Extension, Json};
use ciproxy_store_interface::billing::state::Tier;
use ciproxy_store_interface::dashboard::records::{AdminClaim, Role};
use ciproxy_store_interface::organizations::AppState;

use crate::access::Principal;
use crate::error::{Error, Result};
use crate::state::{ApiState, query};
use crate::wire::access::{
    SessionQuery, SessionView, SignInState, Success, WorkspacePath, WorkspaceView,
};

use super::views;

#[utoipa::path(operation_id = "sessions_session", get, path = "/session", params(SessionQuery), responses((status = 200, body = SessionView)), security(("session" = [])))]
pub(crate) async fn session(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Query(parameters): Query<SessionQuery>,
) -> Result<Json<SessionView>> {
    let personal_installation = if let Some(id) = parameters.installation_id {
        let id = match id.parse::<i64>() {
            Ok(value) => value,
            Err(_) => return Err(Error::InvalidInput),
        };
        state
            .installations
            .personal(id, principal.github_id())
            .await?
    } else {
        None
    };
    let pending_install_requests = state
        .installations
        .pending_requests(principal.github_id())
        .await?;
    let id = principal.id();
    let github_id = principal.github_id();
    if !principal.browser_only() {
        let refresh = query(state.store.clone(), move |store| {
            let memberships = store.dashboard_memberships(id)?;
            let mut refresh = Vec::new();
            for workspace in store.dashboard_workspaces(id)? {
                let organizations = store
                    .organizations(workspace.id)?
                    .into_iter()
                    .filter(|organization| organization.app_state != AppState::Removed)
                    .collect::<Vec<_>>();
                let owns_any = organizations.iter().any(|organization| {
                    memberships.iter().any(|membership| {
                        membership.github_id == organization.github_id && membership.owner
                    })
                });
                let owns_all = organizations.iter().all(|organization| {
                    memberships.iter().any(|membership| {
                        membership.github_id == organization.github_id && membership.owner
                    })
                });
                if owns_any && !owns_all {
                    refresh.push(workspace.id);
                }
            }
            Ok(refresh)
        })
        .await?;
        for workspace in refresh {
            state.access.refresh_workspace_admins(workspace).await?;
        }
    }
    let reauthorization_required = if principal.browser_only() {
        query(state.store.clone(), move |store| {
            store.dashboard_has_access_record(github_id)
        })
        .await?
    } else {
        false
    };
    let browser_only = principal.browser_only();
    let workspaces = query(state.store.clone(), move |store| {
        if browser_only {
            return Ok(Vec::new());
        }
        let memberships = store.dashboard_memberships(id)?;
        let mut result = Vec::new();
        for workspace in store.dashboard_workspaces(id)? {
            let members = store.dashboard_members(workspace.id)?;
            let admins = members
                .iter()
                .filter(|member| member.role == Some(Role::Admin))
                .collect::<Vec<_>>();
            let organizations = store
                .organizations(workspace.id)?
                .into_iter()
                .filter(|organization| organization.app_state != AppState::Removed)
                .collect::<Vec<_>>();
            let owns_any = organizations.iter().any(|organization| {
                memberships.iter().any(|membership| {
                    membership.github_id == organization.github_id && membership.owner
                })
            });
            let owns_all = organizations.iter().all(|organization| {
                memberships.iter().any(|membership| {
                    membership.github_id == organization.github_id && membership.owner
                })
            });
            result.push(WorkspaceView {
                id: workspace.id.to_string(),
                name: workspace.name,
                role: store
                    .dashboard_role(workspace.id, github_id)?
                    .map(views::role),
                admins: admins
                    .iter()
                    .filter_map(|member| member.login.clone())
                    .collect(),
                can_claim_admin: owns_any && (admins.is_empty() || owns_all),
                organization_limit_exceeded: store.billing(workspace.id)?.state.tier == Tier::Free
                    && organizations.len() > 1,
            });
        }
        Ok(result)
    })
    .await?;
    Ok(Json(SessionView {
        personal_installation,
        pending_install_requests,
        removed_organization: principal.removed_organization().cloned(),
        state: if principal.browser_only() {
            if principal.removed_organization().is_some() {
                SignInState::AppRemoved
            } else if principal.had_access() {
                SignInState::AccessEnded
            } else {
                SignInState::InstallRequired
            }
        } else {
            SignInState::Ready
        },
        csrf_token: principal.csrf().to_owned(),
        reauthorization_url: principal
            .browser_only()
            .then(|| "/api/auth/github".to_owned()),
        reauthorization_required,
        login: principal.login().to_owned(),
        expires_at: principal.expires_at(),
        workspaces,
        install_url: state.config.github_install_url,
    }))
}

#[utoipa::path(operation_id = "sessions_claim", post, path = "/workspaces/{workspace_id}/claim-admin", params(WorkspacePath), responses((status = 200, body = Success)), security(("session" = [])))]
pub(crate) async fn claim(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<WorkspacePath>,
) -> Result<Json<Success>> {
    let id = principal.id();
    if principal.browser_only() {
        return Err(Error::Forbidden);
    }
    let visible = query(state.store.clone(), move |store| {
        Ok(store
            .dashboard_workspaces(id)?
            .iter()
            .any(|workspace| workspace.id == path.workspace_id))
    })
    .await?;
    if !visible {
        return Err(Error::Forbidden);
    }
    let now = state.runtime.now();
    if query(state.store.clone(), move |store| {
        store.claim_dashboard_admin(path.workspace_id, id, now, AdminClaim::AllOrganizations)
    })
    .await?
    {
        return Ok(Json(Success { ok: true }));
    }
    state
        .access
        .refresh_workspace_admins(path.workspace_id)
        .await?;
    let now = state.runtime.now();
    let claimed = query(state.store, move |store| {
        store.claim_dashboard_admin(path.workspace_id, id, now, AdminClaim::Recovery)
    })
    .await?;
    if !claimed {
        return Err(Error::Forbidden);
    }
    Ok(Json(Success { ok: true }))
}
