//! Label controls use GitHub acceptance and preserve immutable job history.

use std::error;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::{Extension, Json};
use ciproxy_store_interface::labels::Label;
use ciproxy_store_interface::organizations::AppState;
use ciproxy_store_interface::scale_sets::ScaleSet;
use uuid::Uuid;

use crate::access::Principal;
use crate::error::{Error, ErrorResultExt, Result, ServiceResultExt};
use crate::state::{ApiState, query};
use crate::wire::access::{Success, WorkspacePath};
use crate::wire::settings::{LabelFit, LabelInput, LabelPath, LabelView};

use super::label_views;

pub(super) async fn read(state: &ApiState, workspace: Uuid) -> Result<Vec<LabelView>> {
    let labels = query(state.store.clone(), move |store| store.labels(workspace)).await?;
    let gateway = state.gateway.clone();
    let machines = tokio::task::spawn_blocking(move || gateway.machines(workspace))
        .await
        .internal()?
        .api()?;
    Ok(labels
        .into_iter()
        .map(|label| label_views::label(label, &machines, state.capacity.as_ref()))
        .collect())
}

#[utoipa::path(operation_id = "labels_list", get, path = "/workspaces/{workspace_id}/labels", params(WorkspacePath), responses((status = 200, body = [LabelView])), security(("session" = [])))]
pub(crate) async fn list(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<WorkspacePath>,
) -> Result<Json<Vec<LabelView>>> {
    let workspace_id = state
        .access
        .require_role(&principal, path.workspace_id, true)
        .await?;
    Ok(Json(read(&state, workspace_id).await?))
}

#[utoipa::path(operation_id = "labels_preview", post, path = "/workspaces/{workspace_id}/labels/preview", params(WorkspacePath), request_body = LabelInput, responses((status = 200, body = LabelFit)), security(("session" = [])))]
pub(crate) async fn preview(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<WorkspacePath>,
    Json(input): Json<LabelInput>,
) -> Result<Json<LabelFit>> {
    let workspace_id = state
        .access
        .require_role(&principal, path.workspace_id, true)
        .await?;
    label_views::stored(state.runtime.identifier(), input.clone())?;
    let gateway = state.gateway;
    let machines = tokio::task::spawn_blocking(move || gateway.machines(workspace_id))
        .await
        .internal()?
        .api()?;
    Ok(Json(label_views::fit(
        &input,
        &machines,
        state.capacity.as_ref(),
    )))
}

#[utoipa::path(operation_id = "labels_create", post, path = "/workspaces/{workspace_id}/labels", params(WorkspacePath), request_body = LabelInput, responses((status = 200, body = LabelView)), security(("session" = [])))]
pub(crate) async fn create(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<WorkspacePath>,
    Json(input): Json<LabelInput>,
) -> Result<Json<LabelView>> {
    let workspace_id = state
        .access
        .require_role(&principal, path.workspace_id, true)
        .await?;
    let label = label_views::stored(state.runtime.identifier(), input)?;
    let retired = query(state.store.clone(), move |store| {
        store.retired_dashboard_labels(workspace_id)
    })
    .await?;
    for previous in retired
        .into_iter()
        .filter(|previous| previous.name == label.name)
    {
        cleanup_label(&state, workspace_id, previous.id).await?;
    }
    prepare(&state, workspace_id, &label).await?;
    let saved = label.clone();
    query(state.store.clone(), move |store| {
        store.create_label(workspace_id, &saved)
    })
    .await?;
    for organization in query(state.store.clone(), move |store| {
        store.organizations(workspace_id)
    })
    .await?
    .into_iter()
    .filter(|organization| organization.app_state == AppState::Active)
    {
        state
            .github
            .refresh_repositories(organization.installation_id)
            .await
            .api()?;
    }
    Ok(Json(
        read(&state, workspace_id)
            .await?
            .into_iter()
            .find(|saved| saved.id == label.id.to_string())
            .ok_or(Error::NotFound)?,
    ))
}

#[utoipa::path(operation_id = "labels_update", put, path = "/workspaces/{workspace_id}/labels/{label_id}", params(LabelPath), request_body = LabelInput, responses((status = 200, body = LabelView)), security(("session" = [])))]
pub(crate) async fn update(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<LabelPath>,
    Json(input): Json<LabelInput>,
) -> Result<Json<LabelView>> {
    let workspace_id = state
        .access
        .require_role(&principal, path.workspace_id, true)
        .await?;
    let previous = query(state.store.clone(), move |store| store.labels(workspace_id))
        .await?
        .into_iter()
        .find(|label| label.id == path.label_id)
        .ok_or(Error::NotFound)?;
    if previous.name != input.name {
        return Err(Error::Conflict);
    }
    let label = label_views::stored(path.label_id, input)?;
    query(state.store.clone(), move |store| {
        store.update_label(workspace_id, &label)
    })
    .await?;
    Ok(Json(
        read(&state, workspace_id)
            .await?
            .into_iter()
            .find(|saved| saved.id == path.label_id.to_string())
            .ok_or(Error::NotFound)?,
    ))
}

#[utoipa::path(operation_id = "labels_remove", delete, path = "/workspaces/{workspace_id}/labels/{label_id}", params(LabelPath), responses((status = 200, body = Success)), security(("session" = [])))]
pub(crate) async fn remove(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<LabelPath>,
) -> Result<Json<Success>> {
    let workspace_id = state
        .access
        .require_role(&principal, path.workspace_id, true)
        .await?;
    let at = state.runtime.now();
    query(state.store.clone(), move |store| {
        store.retire_dashboard_label(workspace_id, path.label_id, at)
    })
    .await?;
    cleanup_label(&state, workspace_id, path.label_id).await?;
    Ok(Json(Success { ok: true }))
}

async fn prepare(state: &ApiState, workspace: Uuid, label: &Label) -> Result<()> {
    let organizations = query(state.store.clone(), move |store| {
        store.organizations(workspace)
    })
    .await?;
    let mut checked = false;
    for organization in organizations
        .into_iter()
        .filter(|organization| organization.app_state == AppState::Active)
    {
        let id = organization.github_id;
        let mut lease = query(state.store.clone(), move |store| {
            store.lock_github_organization(id)
        })
        .await?;
        let sets = query(state.store.clone(), move |store| {
            store.scale_sets(workspace, organization.id)
        })
        .await?;
        let group = sets
            .first()
            .map(|set| set.runner_group_id)
            .ok_or(Error::Unavailable)?;
        let api = state
            .scale_sets
            .for_organization(&organization)
            .await
            .api()?;
        match api.ensure(&label.name, group, None).await {
            Ok(_) => checked = true,
            Err(error) => return Err(label_failure(&error)),
        }
        lease.check_connection().internal()?;
    }
    if checked {
        Ok(())
    } else {
        Err(Error::Unavailable)
    }
}

pub(crate) async fn cleanup_label(state: &ApiState, workspace: Uuid, label: Uuid) -> Result<()> {
    for organization in query(state.store.clone(), move |store| {
        store.organizations(workspace)
    })
    .await?
    .into_iter()
    .filter(|organization| organization.app_state == AppState::Active)
    {
        let id = organization.github_id;
        let mut lease = query(state.store.clone(), move |store| {
            store.lock_github_organization(id)
        })
        .await?;
        let organization_id = organization.id;
        let sets: Vec<ScaleSet> = query(state.store.clone(), move |store| {
            store.dashboard_scale_sets(workspace, organization_id)
        })
        .await?;
        let api = state
            .scale_sets
            .for_organization(&organization)
            .await
            .api()?;
        for set in sets.into_iter().filter(|set| set.label_id == label) {
            api.remove_scale_set(set.github_id).await.api()?;
        }
        lease.check_connection().internal()?;
    }
    Ok(())
}

fn label_failure(error: &ciproxy_github::error::Error) -> Error {
    let mut source: Option<&(dyn error::Error + 'static)> = Some(error);
    while let Some(current) = source {
        if let Some(sdk) = current.downcast_ref::<scaleset::Error>()
            && let Some(response) = sdk.api_error()
            && response.method == reqwest::Method::POST
            && reqwest::Url::parse(&response.url)
                .ok()
                .is_some_and(|url| url.path().ends_with("/runnerscalesets"))
            && matches!(response.status.as_u16(), 400 | 422)
            && let Some(exception) = &response.exception
        {
            return Error::LabelRefused {
                message: exception.message.clone(),
            };
        }
        source = current.source();
    }
    Error::Unavailable
}
