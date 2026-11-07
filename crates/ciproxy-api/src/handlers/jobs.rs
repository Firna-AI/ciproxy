//! Current-workspace Jobs live, history, and detail routes.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::{Extension, Json};
use chrono::Duration;
use ciproxy_store_interface::billing::state::Tier;
use ciproxy_store_interface::job_views::DashboardJob;
use ciproxy_store_interface::jobs::JobState;
use uuid::Uuid;

use crate::access::Principal;
use crate::error::{Error, Result};
use crate::pages::PageServices;
use crate::state::{ApiState, query};
use crate::wire::access::WorkspacePath;
use crate::wire::jobs::{
    HistoryPlace, JobDetailPage, JobHistoryQuery, JobPath, JobsHistoryPage, JobsLivePage,
};

use super::{job_views, jobs_live, page_context};

pub(super) fn history_days(tier: Tier) -> u32 {
    match tier {
        Tier::Free => 7,
        Tier::Team => 30,
        Tier::Business => 90,
    }
}

pub(super) async fn read(
    state: &ApiState,
    workspace: Uuid,
    context: &page_context::Context,
) -> Result<Vec<DashboardJob>> {
    let since = context.at - Duration::days(i64::from(history_days(context.record.state.tier)));
    query(state.store.clone(), move |store| {
        store.dashboard_jobs(workspace, since)
    })
    .await
}

#[utoipa::path(operation_id="jobs_live",get,path="/workspaces/{workspace_id}/jobs/live",params(WorkspacePath),responses((status=200,body=JobsLivePage)),security(("session"=[])))]
pub(crate) async fn live(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    pages: Option<Extension<Arc<PageServices>>>,
    Path(path): Path<WorkspacePath>,
) -> Result<Json<JobsLivePage>> {
    let workspace = state
        .access
        .require_role(&principal, path.workspace_id, false)
        .await?;
    let context = page_context::read(&state, workspace, principal.github_id()).await?;
    let jobs = read(&state, workspace, &context).await?;
    Ok(Json(
        jobs_live::page(&state, pages.map(|pages| pages.0), workspace, context, jobs).await?,
    ))
}

#[utoipa::path(operation_id="jobs_history",get,path="/workspaces/{workspace_id}/jobs/history",params(WorkspacePath,JobHistoryQuery),responses((status=200,body=JobsHistoryPage)),security(("session"=[])))]
pub(crate) async fn history(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<WorkspacePath>,
    Query(input): Query<JobHistoryQuery>,
) -> Result<Json<JobsHistoryPage>> {
    let workspace = state
        .access
        .require_role(&principal, path.workspace_id, false)
        .await?;
    let mut context = page_context::read(&state, workspace, principal.github_id()).await?;
    let jobs = read(&state, workspace, &context)
        .await?
        .into_iter()
        .filter(|job| job.state == JobState::Completed)
        .filter(|job| {
            input.place.is_none_or(|place| match place {
                HistoryPlace::Machine => job.machine_id.is_some(),
                HistoryPlace::Boat => job.boat_size.is_some(),
            })
        })
        .filter(|job| {
            input.search.as_ref().is_none_or(|search| {
                let search = search.to_lowercase();
                [&job.name, &job.workflow, &job.repository, &job.organization]
                    .iter()
                    .any(|value| value.to_lowercase().contains(&search))
            })
        })
        .filter(|job| {
            let day = job
                .completed_at
                .or(job.started_at)
                .or(job.queued_at)
                .map(|at| at.date_naive());
            input
                .from
                .is_none_or(|from| day.is_some_and(|day| day >= from))
                && input
                    .until
                    .is_none_or(|until| day.is_some_and(|day| day < until))
        })
        .map(|job| job_views::job(job, context.at))
        .collect();
    Ok(Json(JobsHistoryPage {
        jobs,
        history_days: history_days(context.record.state.tier),
        banner: page_context::banner(&mut context)?,
    }))
}

#[utoipa::path(operation_id="jobs_detail",get,path="/workspaces/{workspace_id}/jobs/{job_id}",params(JobPath),responses((status=200,body=JobDetailPage)),security(("session"=[])))]
pub(crate) async fn detail(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<JobPath>,
) -> Result<Json<JobDetailPage>> {
    let workspace = state
        .access
        .require_role(&principal, path.workspace_id, false)
        .await?;
    let mut context = page_context::read(&state, workspace, principal.github_id()).await?;
    let value = read(&state, workspace, &context)
        .await?
        .into_iter()
        .find(|job| job.id == path.job_id)
        .ok_or(Error::NotFound)?;
    Ok(Json(JobDetailPage {
        job: job_views::job(value, context.at),
        banner: page_context::banner(&mut context)?,
    }))
}
