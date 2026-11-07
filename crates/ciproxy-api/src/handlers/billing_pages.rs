//! Read-only usage, published Plans, and the reusable one-banner route.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::{Extension, Json};
use ciproxy_metering::pricing::{Plan, Tier, plan_choice};

use crate::access::Principal;
use crate::error::Result;
use crate::state::ApiState;
use crate::wire::access::WorkspacePath;
use crate::wire::billing::{PlanAddOns, PlanCard, PlanFeature, PlanUsagePage, PlansPage};
use crate::wire::pages::{PageBannerQuery, PageBannerView};

use super::page_context;

#[utoipa::path(operation_id="plan_usage",get,path="/workspaces/{workspace_id}/billing",params(WorkspacePath),responses((status=200,body=PlanUsagePage)),security(("session"=[])))]
pub(crate) async fn usage(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<WorkspacePath>,
) -> Result<Json<PlanUsagePage>> {
    let workspace = state
        .access
        .require_role(&principal, path.workspace_id, false)
        .await?;
    Ok(Json(page_context::usage(
        page_context::read(&state, workspace, principal.github_id()).await?,
    )?))
}

#[utoipa::path(operation_id="plans",get,path="/workspaces/{workspace_id}/plans",params(WorkspacePath),responses((status=200,body=PlansPage)),security(("session"=[])))]
pub(crate) async fn plans(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<WorkspacePath>,
) -> Result<Json<PlansPage>> {
    let workspace = state
        .access
        .require_role(&principal, path.workspace_id, false)
        .await?;
    let usage =
        page_context::usage(page_context::read(&state, workspace, principal.github_id()).await?)?;
    let mut plans = Vec::new();
    for tier in [Tier::Free, Tier::Team, Tier::Business] {
        let offer = page_context::offer(plan_choice(Plan {
            tier,
            add_ons: Default::default(),
        }))?;
        let current = offer.selection.tier == usage.status.selection.tier;
        let covers_use = offer.selection.tier == usage.offer.selection.tier && !current;
        let mut features = vec![
            feature("Own Linux machines", false),
            feature("Boat overflow", false),
            feature("Transparent cache", false),
            feature("Warm runners", false),
            feature("Savings report", false),
        ];
        if tier != Tier::Free {
            features.extend([
                feature("Several GitHub organizations", false),
                feature("Sticky Docker disks", false),
                feature("Email and Slack alerts", true),
                feature("Email support", false),
            ]);
        }
        if tier == Tier::Business {
            features.extend([
                feature("Single sign-on", true),
                feature("Audit log", true),
                feature("Custom VM images", true),
                feature("Priority support", false),
            ]);
        }
        plans.push(PlanCard {
            can_choose: usage.status.can_pay
                && !current
                && !usage.status.payment_pending
                && !usage.status.operation_in_progress,
            current,
            covers_use,
            history_days: match tier {
                Tier::Free => 7,
                Tier::Team => 30,
                Tier::Business => 90,
            },
            organizations: (tier == Tier::Free).then_some(1),
            features,
            initial_add_ons: if offer.selection.tier == usage.offer.selection.tier {
                usage.offer.selection.add_ons
            } else {
                PlanAddOns::default()
            },
            plan: offer,
        });
    }
    Ok(Json(PlansPage { usage, plans }))
}

fn feature(name: &str, coming_later: bool) -> PlanFeature {
    PlanFeature {
        name: name.to_owned(),
        coming_later,
    }
}

#[utoipa::path(operation_id="page_banner",get,path="/workspaces/{workspace_id}/banner",params(WorkspacePath,PageBannerQuery),responses((status=200,body=PageBannerView)),security(("session"=[])))]
pub(crate) async fn banner(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    Path(path): Path<WorkspacePath>,
    Query(input): Query<PageBannerQuery>,
) -> Result<Json<PageBannerView>> {
    let workspace = state
        .access
        .require_role(&principal, path.workspace_id, false)
        .await?;
    let mut context = page_context::read(&state, workspace, principal.github_id()).await?;
    let banner = if input.page.is_some() {
        page_context::organizations_banner(&mut context)?
    } else {
        page_context::banner(&mut context)?
    };
    Ok(Json(PageBannerView { banner }))
}
