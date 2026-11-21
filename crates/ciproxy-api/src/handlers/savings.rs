//! Workspace-wide report reads through the existing Metering service.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::{Extension, Json};
use ciproxy_metering::report::range::{ReportRange, report_range};
use ciproxy_store_interface::dashboard::records::Role;

use crate::access::Principal;
use crate::error::{Error, Result};
use crate::pages::PageServices;
use crate::state::{ApiState, query};
use crate::wire::access::WorkspacePath;
use crate::wire::savings::{SavingsPage, SavingsQuery, SavingsRange};

use super::{page_context, savings_views};

#[utoipa::path(operation_id="savings",get,path="/workspaces/{workspace_id}/savings",params(WorkspacePath,SavingsQuery),responses((status=200,body=SavingsPage)),security(("session"=[])))]
pub(crate) async fn report(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    pages: Option<Extension<Arc<PageServices>>>,
    Path(path): Path<WorkspacePath>,
    Query(input): Query<SavingsQuery>,
) -> Result<Json<SavingsPage>> {
    let workspace = state
        .access
        .require_role(&principal, path.workspace_id, false)
        .await?;
    let mut context = page_context::read(&state, workspace, principal.github_id()).await?;
    let pages = pages.ok_or(Error::Unavailable)?.0;
    let lists = pages.price_lists.clone().ok_or(Error::Unavailable)?;
    let selection = match input.range {
        SavingsRange::Last30Days => ReportRange::Last30Days,
        SavingsRange::ThisMonth => ReportRange::ThisMonth,
        SavingsRange::LastMonth => ReportRange::LastMonth,
    };
    let range = report_range(selection, context.at).ok_or(Error::InvalidInput)?;
    let at = context.at;
    let (report, refresh) = query(state.store.clone(), move |_| {
        let refresh = pages.metering.refresh(workspace, at)?;
        Ok((
            pages.metering.savings(workspace, range, at, &lists)?,
            refresh,
        ))
    })
    .await?;
    let mut page = savings_views::report(
        report,
        input.range,
        refresh.stale_jobs == 0 && refresh.stale_sandboxes == 0,
    );
    page.can_add_machine_costs = context.role == Role::Admin;
    page.banner = page_context::banner(&mut context)?;
    Ok(Json(page))
}
