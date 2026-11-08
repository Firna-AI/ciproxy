//! Setup and Admin-only non-Boat workspace settings.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::{Extension, Json};
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::runners::RunnerPlace;

use crate::access::Principal;
use crate::error::{Error, Result};
use crate::pages::PageServices;
use crate::state::{ApiState, query};
use crate::wire::access::{Success, WorkspacePath};
use crate::wire::settings::{
    SettingsView, SetupJob, SetupPhase, SetupPlace, SetupView, WorkspaceName,
};

use super::{labels, views};

#[utoipa::path(operation_id = "setup_setup", get, path = "/workspaces/{workspace_id}/setup", params(WorkspacePath), responses((status = 200, body = SetupView)), security(("session" = [])))]
pub(crate) async fn setup(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<WorkspacePath>,
    pages: Option<Extension<Arc<PageServices>>>,
) -> Result<Json<SetupView>> {
    let workspace_id = state
        .access
        .require_role(&principal, path.workspace_id, true)
        .await?;
    let organizations = query(state.store.clone(), move |store| {
        views::organizations(store, workspace_id)
    })
    .await?;
    let machines = query(state.store.clone(), move |store| {
        store.machines(workspace_id)
    })
    .await?
    .len()
    .try_into()
    .unwrap_or(u32::MAX);
    let first_job = query(state.store.clone(), move |store| {
        let mut jobs = Vec::new();
        for organization in store.organizations(workspace_id)? {
            for job in store.github_jobs(workspace_id, organization.id)? {
                let place = if let Some(github_id) = job.runner_github_id {
                    store
                        .runners(workspace_id, organization.id)?
                        .into_iter()
                        .find(|runner| {
                            runner.organization_id == organization.id
                                && runner.scale_set_id == job.scale_set_id
                                && runner.github_id == Some(github_id)
                        })
                        .map(|runner| match runner.place {
                            RunnerPlace::Machine { .. } => SetupPlace::Machine,
                            RunnerPlace::Boat { .. } => SetupPlace::Boat,
                        })
                } else {
                    None
                };
                let phase = match job.phase {
                    DemandPhase::Available => SetupPhase::Available,
                    DemandPhase::Assigned => SetupPhase::Assigned,
                    DemandPhase::Started => SetupPhase::Started,
                    DemandPhase::Completed => SetupPhase::Completed,
                };
                jobs.push(SetupJob {
                    id: job.id.to_string(),
                    name: job.name,
                    repository: job.repository,
                    phase,
                    workflow_ref: job.workflow_ref,
                    result: job.result,
                    place,
                    assigned_at: job.assigned_at,
                    started_at: job.started_at,
                    completed_at: job.completed_at,
                });
            }
        }
        jobs.sort_by_key(|job| (job.assigned_at.is_none(), job.assigned_at, job.id.clone()));
        Ok(jobs.into_iter().next())
    })
    .await?;
    Ok(Json(SetupView {
        current_image_disk_bytes: state.config.image_disk_bytes.to_string(),
        first_job,
        organizations,
        machines,
        waiting_for_machine: machines == 0,
        labels: labels::read(
            &state,
            workspace_id,
            pages.as_ref().map(|pages| pages.0.as_ref()),
        )
        .await?,
    }))
}

#[utoipa::path(operation_id = "setup_settings", get, path = "/workspaces/{workspace_id}/settings", params(WorkspacePath), responses((status = 200, body = SettingsView)), security(("session" = [])))]
pub(crate) async fn settings(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<WorkspacePath>,
    pages: Option<Extension<Arc<PageServices>>>,
) -> Result<Json<SettingsView>> {
    let workspace_id = state
        .access
        .require_role(&principal, path.workspace_id, true)
        .await?;
    let (workspace, organizations, requests) = query(state.store.clone(), move |store| {
        Ok((
            store.workspace(workspace_id)?,
            views::organizations(store, workspace_id)?,
            store
                .dashboard_members(workspace_id)?
                .iter()
                .filter(|member| member.role.is_none())
                .count(),
        ))
    })
    .await?;
    let workspace = workspace.ok_or(Error::NotFound)?;
    Ok(Json(SettingsView {
        name: workspace.name,
        organizations,
        labels: labels::read(
            &state,
            workspace_id,
            pages.as_ref().map(|pages| pages.0.as_ref()),
        )
        .await?,
        open_requests: requests.try_into().unwrap_or(u32::MAX),
        can_add_organization: true,
    }))
}

#[utoipa::path(operation_id = "setup_rename", put, path = "/workspaces/{workspace_id}/settings", params(WorkspacePath), request_body = WorkspaceName, responses((status = 200, body = Success)), security(("session" = [])))]
pub(crate) async fn rename(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<WorkspacePath>,
    Json(body): Json<WorkspaceName>,
) -> Result<Json<Success>> {
    let workspace_id = state
        .access
        .require_role(&principal, path.workspace_id, true)
        .await?;
    query(state.store, move |store| {
        store.rename_workspace(workspace_id, &body.name)
    })
    .await?;
    Ok(Json(Success { ok: true }))
}
