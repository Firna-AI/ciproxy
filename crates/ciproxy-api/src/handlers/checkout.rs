//! Authorized Checkout review, payment, return, and paid-plan commands.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::{Extension, Json};
use ciproxy_metering::pricing::plan_choice;

use crate::access::Principal;
use crate::error::{Error, ErrorResultExt, Result};
use crate::pages::PageServices;
use crate::state::ApiState;
use crate::wire::access::{Success, WorkspacePath};
use crate::wire::billing::{
    CheckoutReturn, CheckoutReturnPage, CheckoutReviewPage, CheckoutReviewQuery, PaymentOutcome,
    PlanAddOns, PlanSelection, PlanTier, PortalLink,
};
use crate::wire::money::Cents;

use super::{billing_call, billing_views, page_context};

async fn context(
    state: &ApiState,
    principal: &Principal,
    path: WorkspacePath,
) -> Result<(uuid::Uuid, page_context::Context)> {
    let workspace = state
        .access
        .require_role(principal, path.workspace_id, false)
        .await?;
    let context = page_context::read(state, workspace, principal.github_id()).await?;
    if !page_context::payer(context.role) {
        return Err(Error::Forbidden);
    }
    Ok((workspace, context))
}

fn services(value: Option<Extension<Arc<PageServices>>>) -> Result<Arc<PageServices>> {
    value.map(|value| value.0).ok_or(Error::Unavailable)
}

#[utoipa::path(operation_id="checkout_review",get,path="/workspaces/{workspace_id}/checkout/review",params(WorkspacePath,CheckoutReviewQuery),responses((status=200,body=CheckoutReviewPage)),security(("session"=[])))]
pub(crate) async fn review(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    pages: Option<Extension<Arc<PageServices>>>,
    Path(path): Path<WorkspacePath>,
    Query(input): Query<CheckoutReviewQuery>,
) -> Result<Json<CheckoutReviewPage>> {
    let (workspace, mut context) = context(&state, &principal, path).await?;
    if context.record.command.is_some() || context.record.checkout.is_some() {
        return Err(Error::Conflict);
    }
    let suggested = billing_views::choice_selection(context.usage.offer.plan)?;
    let tier = input.tier.unwrap_or(suggested.tier);
    let defaults = if tier == suggested.tier {
        suggested.add_ons
    } else {
        PlanAddOns::default()
    };
    let selected = PlanSelection {
        tier,
        add_ons: PlanAddOns {
            start_blocks: input.start_blocks.unwrap_or(defaults.start_blocks),
            machines: input.machines.unwrap_or(defaults.machines),
        },
    };
    if tier == PlanTier::Free || tier == billing_views::tier(context.record.state.tier) {
        return Err(Error::Conflict);
    }
    let selected = billing_views::stored(selected);
    let access = billing_views::access(workspace, context.role);
    let pages = services(pages)?;
    let prorated = context.record.state.tier != ciproxy_store_interface::billing::state::Tier::Free;
    let review = if prorated {
        let provider = pages.paid_review.clone().ok_or(Error::Unavailable)?;
        let at = context.at;
        billing_call::result(
            tokio::task::spawn_blocking(move || provider.review(access, selected, at))
                .await
                .internal()?,
        )?
    } else {
        billing_call::call(pages, move |billing| billing.review(access, selected)).await?
    };
    let choice = page_context::offer(plan_choice(billing_views::plan(
        selected.tier,
        selected.add_ons,
    )))?;
    Ok(Json(CheckoutReviewPage {
        choice,
        today_cents: Cents(i128::from(review.today_cents)),
        billing_day: review.billing_day,
        prorated,
        period_end: context.period.ends_at,
        banner: page_context::banner(&mut context)?,
    }))
}

#[utoipa::path(operation_id="checkout_open",post,path="/workspaces/{workspace_id}/checkout",params(WorkspacePath),request_body=PlanSelection,responses((status=200,body=PaymentOutcome)),security(("session"=[])))]
pub(crate) async fn open(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    pages: Option<Extension<Arc<PageServices>>>,
    Path(path): Path<WorkspacePath>,
    Json(input): Json<PlanSelection>,
) -> Result<Json<PaymentOutcome>> {
    let (workspace, context) = context(&state, &principal, path).await?;
    let access = billing_views::access(workspace, context.role);
    Ok(Json(billing_views::checkout(
        billing_call::call(services(pages)?, move |billing| {
            billing.open_checkout(access, billing_views::stored(input))
        })
        .await?,
    )))
}

#[utoipa::path(operation_id="checkout_return",post,path="/workspaces/{workspace_id}/checkout/return",params(WorkspacePath),request_body=CheckoutReturn,responses((status=200,body=CheckoutReturnPage)),security(("session"=[])))]
pub(crate) async fn returned(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    pages: Option<Extension<Arc<PageServices>>>,
    Path(path): Path<WorkspacePath>,
    Json(input): Json<CheckoutReturn>,
) -> Result<Json<CheckoutReturnPage>> {
    let (workspace, context) = context(&state, &principal, path).await?;
    let access = billing_views::access(workspace, context.role);
    let outcome = billing_views::checkout(
        billing_call::call(services(pages)?, move |billing| {
            billing.reconcile_checkout(access, &input.session_id)
        })
        .await?,
    );
    let mut current = page_context::read(&state, workspace, principal.github_id()).await?;
    Ok(Json(CheckoutReturnPage {
        outcome,
        banner: page_context::banner(&mut current)?,
    }))
}

#[utoipa::path(operation_id="billing_change",post,path="/workspaces/{workspace_id}/billing/change",params(WorkspacePath),request_body=PlanSelection,responses((status=200,body=PaymentOutcome)),security(("session"=[])))]
pub(crate) async fn change(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    pages: Option<Extension<Arc<PageServices>>>,
    Path(path): Path<WorkspacePath>,
    Json(input): Json<PlanSelection>,
) -> Result<Json<PaymentOutcome>> {
    let (workspace, context) = context(&state, &principal, path).await?;
    let access = billing_views::access(workspace, context.role);
    if input.tier == billing_views::tier(context.record.state.tier) {
        return Err(Error::Conflict);
    }
    Ok(Json(billing_views::change(
        billing_call::call(services(pages)?, move |billing| {
            billing.change(access, billing_views::stored(input))
        })
        .await?,
    )))
}

#[utoipa::path(operation_id="billing_addons",put,path="/workspaces/{workspace_id}/billing/add-ons",params(WorkspacePath),request_body=PlanAddOns,responses((status=200,body=PaymentOutcome)),security(("session"=[])))]
pub(crate) async fn addons(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    pages: Option<Extension<Arc<PageServices>>>,
    Path(path): Path<WorkspacePath>,
    Json(input): Json<PlanAddOns>,
) -> Result<Json<PaymentOutcome>> {
    let (workspace, context) = context(&state, &principal, path).await?;
    let access = billing_views::access(workspace, context.role);
    let selected = billing_views::stored(PlanSelection {
        tier: billing_views::tier(context.record.state.tier),
        add_ons: input,
    });
    Ok(Json(billing_views::change(
        billing_call::call(services(pages)?, move |billing| {
            billing.change(access, selected)
        })
        .await?,
    )))
}

#[utoipa::path(operation_id="billing_move_to_free",post,path="/workspaces/{workspace_id}/billing/move-to-free",params(WorkspacePath),responses((status=200,body=Success)),security(("session"=[])))]
pub(crate) async fn free(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    pages: Option<Extension<Arc<PageServices>>>,
    Path(path): Path<WorkspacePath>,
) -> Result<Json<Success>> {
    let (workspace, context) = context(&state, &principal, path).await?;
    let access = billing_views::access(workspace, context.role);
    billing_call::call(services(pages)?, move |billing| {
        billing.move_to_free(access)
    })
    .await?;
    Ok(Json(Success { ok: true }))
}

#[utoipa::path(operation_id="billing_portal",post,path="/workspaces/{workspace_id}/billing/portal",params(WorkspacePath),responses((status=200,body=PortalLink)),security(("session"=[])))]
pub(crate) async fn portal(
    State(state): State<ApiState>,
    Extension(principal): Extension<Arc<Principal>>,
    pages: Option<Extension<Arc<PageServices>>>,
    Path(path): Path<WorkspacePath>,
) -> Result<Json<PortalLink>> {
    let (workspace, context) = context(&state, &principal, path).await?;
    let access = billing_views::access(workspace, context.role);
    Ok(Json(PortalLink {
        url: billing_call::call(services(pages)?, move |billing| billing.portal(access)).await?,
    }))
}
