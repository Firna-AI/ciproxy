//! Workspace access, pending requests, and owner recovery through HTTP.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::{Extension, Json};
use ciproxy_store_interface::billing::state::Tier;
use ciproxy_store_interface::dashboard::records::{AdminClaim, Role};
use ciproxy_store_interface::organizations::AppState;

use crate::access::Principal;
use crate::error::{Error, Result};
use crate::state::{ApiState, query};
use crate::wire::access::{SessionView, SignInState, Success, WorkspacePath, WorkspaceView};
use crate::wire::links::OrganizationLinkView;

use super::{link_views, links, views};

#[utoipa::path(operation_id = "sessions_session", get, path = "/session", responses((status = 200, body = SessionView)), security(("session" = [])))]
pub(crate) async fn session(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
) -> Result<Json<SessionView>> {
    let pending_install_requests = state
        .installations
        .pending_requests(principal.github_id())
        .await?;
    let id = principal.id();
    let github_id = principal.github_id();
    let durable = !principal.browser_only();
    let organization_links: Vec<OrganizationLinkView> = query(state.store.clone(), move |store| {
        store
            .installation_claims(github_id)?
            .into_iter()
            .map(|claim| link_views::view(store, claim, github_id, durable))
            .collect()
    })
    .await?;
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
        installation_continuation: principal
            .installation_continuation()
            .iter()
            .copied()
            .filter(|id| organization_links.iter().any(|link| link.id == *id))
            .collect(),
        organization_links,
        setup: principal.setup().cloned(),
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
        install_url: links::continuation_link(&state, &principal).await?.url,
    }))
}

#[utoipa::path(operation_id = "sessions_claim", post, path = "/workspaces/{workspace_id}/claim-admin", params(WorkspacePath), responses((status = 200, body = Success)), security(("session" = [])))]
pub(crate) async fn claim(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<WorkspacePath>,
) -> Result<Json<Success>> {
    let id = principal.id();
    let workspace_id = state
        .access
        .open_claim_workspace(&principal, path.workspace_id)
        .await?;
    let now = state.runtime.now();
    if query(state.store.clone(), move |store| {
        store.claim_dashboard_admin(workspace_id, id, now, AdminClaim::AllOrganizations)
    })
    .await?
    {
        return Ok(Json(Success { ok: true }));
    }
    state.access.refresh_workspace_admins(workspace_id).await?;
    let now = state.runtime.now();
    let claimed = query(state.store, move |store| {
        store.claim_dashboard_admin(workspace_id, id, now, AdminClaim::Recovery)
    })
    .await?;
    if !claimed {
        return Err(Error::Forbidden);
    }
    Ok(Json(Success { ok: true }))
}
