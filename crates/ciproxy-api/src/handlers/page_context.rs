//! Shared current-role, billing-period, usage, and single-banner reads.

use chrono::{DateTime, Utc};
use ciproxy_metering::period::free_period;
use ciproxy_metering::pricing::{PlanChoice, plan_choice};
use ciproxy_metering::usage::{Banner, PaymentNotice, Usage, workspace_usage};
use ciproxy_store_interface::billing::commands::CommandKind;
use ciproxy_store_interface::billing::state::SubscriptionState;
use ciproxy_store_interface::billing::store::BillingRecord;
use ciproxy_store_interface::dashboard::records::Role;
use ciproxy_store_interface::metering::history::Period;
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::state::{ApiState, query};
use crate::wire::billing::{PlanOffer, PlanStatus, PlanUsagePage, SubscriptionStatus};
use crate::wire::pages::{BannerAction, InactiveOrganization, PageBanner};

use super::{billing_views, link_views};

pub(super) struct Context {
    pub record: BillingRecord,
    pub period: Period,
    pub role: Role,
    pub usage: Usage,
    pub inactive: Vec<InactiveOrganization>,
    pub at: DateTime<Utc>,
}

pub(super) async fn read(state: &ApiState, workspace: Uuid, person: i64) -> Result<Context> {
    let at = state.runtime.now();
    query(state.store.clone(), move |store| {
        let role = store
            .dashboard_role(workspace, person)?
            .ok_or(ciproxy_store_interface::error::Error::NotFound)?;
        let record = store.billing(workspace)?;
        let saved = &record.state;
        let period = if saved.tier == ciproxy_store_interface::billing::state::Tier::Free {
            free_period(saved.free_started_at, at)
        } else {
            saved
                .paid_period_start
                .zip(saved.paid_period_end)
                .map(|(starts_at, ends_at)| Period { starts_at, ends_at })
        }
        .ok_or(ciproxy_store_interface::error::Error::InvalidValue)?;
        let payment = if saved.status == SubscriptionState::PaymentFailed {
            PaymentNotice::Failed
        } else if saved.ended_banner_until.is_some_and(|until| at < until) {
            PaymentNotice::Ended
        } else {
            PaymentNotice::None
        };
        let usage = workspace_usage(
            &store.workspace_usage(workspace)?,
            period,
            billing_views::plan(saved.tier, saved.add_ons),
            at,
            payment,
        );
        let mut inactive = Vec::new();
        for organization in store.organizations(workspace)? {
            if organization.removed_at.is_some() {
                continue;
            }
            let local = store.organization_activation(workspace, organization.id)?;
            if local.billing_hold || local.admin_approval_hold {
                inactive.push(InactiveOrganization {
                    id: organization.id.to_string(),
                    login: organization.login,
                    activation: link_views::activation(local, workspace, Some(role)),
                });
            }
        }
        Ok(Context {
            record,
            period,
            role,
            usage,
            inactive,
            at,
        })
    })
    .await
}

pub(super) fn payer(role: Role) -> bool {
    matches!(role, Role::Admin | Role::Billing)
}

pub(super) fn banner(context: &mut Context) -> Result<Option<PageBanner>> {
    let pay = payer(context.role);
    let action = if pay {
        BannerAction::ChoosePlan
    } else {
        BannerAction::AskAdminOrBilling
    };
    Ok(match context.usage.banner {
        Some(Banner::PaymentFailed) => Some(PageBanner::PaymentFailed {
            failed_at: context.record.state.failed_at,
            next_retry_at: context.record.state.next_retry_at,
            action: if pay {
                BannerAction::UpdateCard
            } else {
                action
            },
        }),
        Some(Banner::SubscriptionEnded) => Some(PageBanner::SubscriptionEnded {
            until: context
                .record
                .state
                .ended_banner_until
                .ok_or(Error::Unavailable)?,
            action,
        }),
        Some(Banner::SoftLimit) => Some(PageBanner::SoftLimit {
            offer: offer(context.usage.offer.clone())?,
            action,
        }),
        _ => None,
    })
}

pub(super) fn organizations_banner(context: &mut Context) -> Result<Option<PageBanner>> {
    if matches!(
        context.usage.banner,
        Some(Banner::PaymentFailed | Banner::SubscriptionEnded)
    ) {
        return banner(context);
    }
    if !context.inactive.is_empty() {
        return Ok(Some(PageBanner::InactiveOrganizations {
            organizations: std::mem::take(&mut context.inactive),
        }));
    }
    banner(context)
}

pub(super) fn offer(choice: PlanChoice) -> Result<PlanOffer> {
    Ok(PlanOffer {
        selection: billing_views::choice_selection(choice.plan)?,
        included_starts: choice.starts.to_string(),
        included_machines: choice.machines.to_string(),
        monthly_cents: billing_views::cents(choice.monthly_usd),
    })
}

pub(super) fn usage(mut context: Context) -> Result<PlanUsagePage> {
    let banner = banner(&mut context)?;
    let state = context.record.state;
    let current = offer(plan_choice(billing_views::plan(state.tier, state.add_ons)))?;
    Ok(PlanUsagePage {
        status: PlanStatus {
            selection: billing_views::selection(state.tier, state.add_ons),
            subscription: match state.status {
                SubscriptionState::None => SubscriptionStatus::None,
                SubscriptionState::Active => SubscriptionStatus::Active,
                SubscriptionState::PaymentFailed => SubscriptionStatus::PaymentFailed,
                SubscriptionState::Ended => SubscriptionStatus::Ended,
            },
            period_start: context.period.starts_at,
            period_end: context.period.ends_at,
            cancel_at: state.cancel_at,
            payment_pending: context.record.checkout.is_some()
                || context.record.command.as_ref().is_some_and(|command| {
                    matches!(
                        command.kind,
                        CommandKind::Checkout { .. } | CommandKind::Change { .. }
                    )
                }),
            operation_in_progress: context.record.command.is_some(),
            can_pay: payer(context.role),
        },
        current,
        starts: context.usage.starts.to_string(),
        machines: context.usage.machines.to_string(),
        organizations: context.usage.organizations.to_string(),
        expected_starts: context.usage.expected_starts.to_string(),
        starts_over: context.usage.starts_over.to_string(),
        machines_over: context.usage.machines_over.to_string(),
        organizations_over: context.usage.organizations_over.to_string(),
        crossing_day: context.usage.crossing_day,
        offer: offer(context.usage.offer)?,
        banner,
    })
}
