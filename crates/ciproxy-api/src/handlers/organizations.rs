//! Organization-specific repository access and restart-safe intentional take-out.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::{Extension, Json};
use ciproxy_store_interface::dashboard::records::Takeout;
use ciproxy_store_interface::organizations::RepositoryAccess;

use crate::access::Principal;
use crate::error::{Error, ErrorResultExt, Result, ServiceResultExt};
use crate::state::{ApiState, query};
use crate::wire::access::{Success, WorkspacePath};
use crate::wire::settings::{
    OrganizationPath, OrganizationView, RepositorySelection, RepositoryView,
};

use super::views;

#[utoipa::path(operation_id = "organizations_list", get, path = "/workspaces/{workspace_id}/organizations", params(WorkspacePath), responses((status = 200, body = [OrganizationView])), security(("session" = [])))]
pub(crate) async fn list(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<WorkspacePath>,
) -> Result<Json<Vec<OrganizationView>>> {
    let workspace_id = state
        .access
        .require_role(&principal, path.workspace_id, true)
        .await?;
    Ok(Json(
        query(state.store, move |store| store.organizations(workspace_id))
            .await?
            .into_iter()
            .map(views::organization)
            .collect(),
    ))
}

#[utoipa::path(operation_id = "organizations_repositories", get, path = "/workspaces/{workspace_id}/organizations/{organization_id}/repositories", params(OrganizationPath), responses((status = 200, body = [RepositoryView])), security(("session" = [])))]
pub(crate) async fn repositories(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<OrganizationPath>,
) -> Result<Json<Vec<RepositoryView>>> {
    let workspace_id = state
        .access
        .require_role(&principal, path.workspace_id, true)
        .await?;
    let organization = query(state.store.clone(), move |store| {
        store.organization(workspace_id, path.organization_id)
    })
    .await?
    .ok_or(Error::NotFound)?;
    state
        .github
        .refresh_repositories(organization.installation_id)
        .await
        .api()?;
    let repositories = query(state.store, move |store| {
        store.repositories(workspace_id, path.organization_id)
    })
    .await?;
    Ok(Json(
        repositories
            .into_iter()
            .map(|repository| {
                let selected = match &organization.repositories {
                    RepositoryAccess::AllPrivate => true,
                    RepositoryAccess::Selected(ids) => ids.contains(&repository.id),
                };
                RepositoryView {
                    id: repository.id.to_string(),
                    full_name: repository.full_name,
                    private: repository.private,
                    allowed: repository.private && selected,
                }
            })
            .collect(),
    ))
}

#[utoipa::path(operation_id = "organizations_save_repositories", put, path = "/workspaces/{workspace_id}/organizations/{organization_id}/repositories", params(OrganizationPath), request_body = RepositorySelection, responses((status = 200, body = Success)), security(("session" = [])))]
pub(crate) async fn save_repositories(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<OrganizationPath>,
    Json(body): Json<RepositorySelection>,
) -> Result<Json<Success>> {
    let workspace_id = state
        .access
        .require_role(&principal, path.workspace_id, true)
        .await?;
    let organization = query(state.store, move |store| {
        store.organization(workspace_id, path.organization_id)
    })
    .await?
    .ok_or(Error::NotFound)?;
    state
        .github
        .set_repository_access(organization.installation_id, views::access(body)?)
        .await
        .api()?;
    Ok(Json(Success { ok: true }))
}

#[utoipa::path(operation_id = "organizations_takeout", delete, path = "/workspaces/{workspace_id}/organizations/{organization_id}", params(OrganizationPath), responses((status = 200, body = Success)), security(("session" = [])))]
pub(crate) async fn takeout(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<OrganizationPath>,
) -> Result<Json<Success>> {
    let workspace_id = state
        .access
        .require_role(&principal, path.workspace_id, true)
        .await?;
    let organization = query(state.store.clone(), move |store| {
        store.organization(workspace_id, path.organization_id)
    })
    .await?
    .ok_or(Error::NotFound)?;
    let github_id = organization.github_id;
    let removal = {
        let mut lease = query(state.store.clone(), move |store| {
            store.lock_github_organization(github_id)
        })
        .await?;
        let removal = query(state.store.clone(), move |store| {
            store.begin_organization_takeout(workspace_id, path.organization_id)
        })
        .await?;
        lease.check_connection().internal()?;
        removal
    };
    state.github.replay_lifecycle(github_id).await.api()?;
    {
        let mut lease = query(state.store.clone(), move |store| {
            store.lock_github_organization(github_id)
        })
        .await?;
        finish_takeout(&state, &removal).await?;
        lease.check_connection().internal()?;
    }
    Ok(Json(Success { ok: true }))
}

pub(crate) async fn finish_takeout(state: &ApiState, removal: &Takeout) -> Result<()> {
    let workspace = removal.workspace_id;
    let id = removal.organization_id;
    let organization = query(state.store.clone(), move |store| {
        store.organization(workspace, id)
    })
    .await?
    .ok_or(Error::NotFound)?;
    if organization.installation_id != removal.installation_id {
        return Err(Error::Conflict);
    }
    if !state.installations.exists(removal.installation_id).await? {
        let saved = removal.clone();
        query(state.store.clone(), move |store| {
            store.complete_organization_takeout(&saved)
        })
        .await?;
        return Ok(());
    }
    let api = state
        .scale_sets
        .for_organization(&organization)
        .await
        .api()?;
    let sets = query(state.store.clone(), move |store| {
        store.dashboard_scale_sets(workspace, id)
    })
    .await?;
    for scale_set in sets {
        api.remove_scale_set(scale_set.github_id).await.api()?;
    }
    state
        .installations
        .uninstall(removal.installation_id)
        .await?;
    let saved = removal.clone();
    query(state.store.clone(), move |store| {
        store.complete_organization_takeout(&saved)
    })
    .await?;
    Ok(())
}
