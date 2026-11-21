//! Live queue figures from gateway reports, runner records, and overflow views.

use std::sync::Arc;

use chrono::{Datelike, Months, NaiveTime};
use ciproxy_gateway::types::Presence;
use ciproxy_protocol::reports::SlotOccupancy;
use ciproxy_store_interface::dashboard::records::Role;
use ciproxy_store_interface::job_views::DashboardJob;
use ciproxy_store_interface::jobs::JobState;
use ciproxy_store_interface::organizations::AppState;
use ciproxy_store_interface::runners::{RunnerPlace, RunnerState};
use ciproxy_store_interface::settings::OverflowWait;
use uuid::Uuid;

use crate::error::{Error, ErrorResultExt, Result, ServiceResultExt};
use crate::pages::PageServices;
use crate::state::{ApiState, query};
use crate::wire::job_overflow::JobOverflow;
use crate::wire::jobs::{JobVisibility, JobsLivePage, StartingRunner};

use super::{billing_views, job_fit, job_views, page_context};

pub(super) async fn page(
    state: &ApiState,
    pages: Option<Arc<PageServices>>,
    workspace: Uuid,
    mut context: page_context::Context,
    jobs: Vec<DashboardJob>,
) -> Result<JobsLivePage> {
    let organizations = context.organizations.clone();
    let (organizations, settings, runners, labels) = query(state.store.clone(), move |store| {
        let mut runners = Vec::new();
        for organization in &organizations {
            runners.extend(store.runners(workspace, organization.id)?);
        }
        Ok((
            organizations,
            store.settings(workspace)?,
            runners,
            store.labels(workspace)?,
        ))
    })
    .await?;
    let complete_waiting = !organizations.iter().any(|organization| {
        context.current_members.contains(&organization.id)
            && organization.app_state == AppState::Suspended
    });
    let visibility: Vec<_> = organizations
        .into_iter()
        .filter(|organization| {
            organization.app_state == AppState::Suspended
                || (organization.app_state == AppState::Removed
                    && organization.removed_at.is_some())
        })
        .map(|organization| JobVisibility {
            organization: organization.login.clone(),
            suspended: organization.app_state == AppState::Suspended,
            removed: organization.app_state == AppState::Removed,
            github_url: if organization.app_state == AppState::Removed {
                state.config.github_install_url.clone()
            } else {
                format!(
                    "https://github.com/organizations/{}/settings/installations/{}",
                    organization.login, organization.installation_id
                )
            },
        })
        .collect();
    let waiting_jobs: Vec<_> = jobs
        .iter()
        .filter(|job| {
            job.state == JobState::Queued && context.current_members.contains(&job.organization_id)
        })
        .cloned()
        .collect();
    let gateway = state.gateway.clone();
    let machines = tokio::task::spawn_blocking(move || gateway.machines(workspace))
        .await
        .internal()?
        .api()?;
    let mut slots = Some(0u64);
    let mut used = Some(0u64);
    for machine in &machines {
        if machine.presence == Presence::Offline {
            slots = None;
            used = None;
            continue;
        }
        let capacity = state.capacity.capacity(&machine.snapshot);
        slots = slots.map(|sum| sum + capacity.slots.0);
        if let Some(report) = &machine.snapshot.report {
            used = used.map(|sum| {
                sum + report
                    .report
                    .slots
                    .iter()
                    .filter(|slot| matches!(slot.occupancy, SlotOccupancy::Job { .. }))
                    .count() as u64
            });
        } else {
            used = None;
        }
    }
    let warm_runners = runners
        .iter()
        .filter(|runner| {
            runner.state == RunnerState::Ready
                && matches!(runner.place, RunnerPlace::Machine { .. })
        })
        .count()
        .to_string();
    let starting_runners = runners
        .into_iter()
        .filter(|runner| runner.state == RunnerState::Starting)
        .map(|runner| StartingRunner {
            id: runner.id.to_string(),
            label_id: runner.label_id.to_string(),
            machine_id: match runner.place {
                RunnerPlace::Machine { machine_id, .. } => Some(machine_id.to_string()),
                _ => None,
            },
            boat: matches!(runner.place, RunnerPlace::Boat { .. }),
        })
        .collect();
    let sizing = pages.as_ref().and_then(|pages| pages.boat_root_sizing);
    let overflow_view = match pages.and_then(|pages| pages.overflow.clone()) {
        Some(service) => service.view(workspace).await.ok(),
        None => None,
    };
    let waiting_labels = job_fit::labels(job_fit::FitInput {
        jobs: &waiting_jobs,
        labels,
        machines: &machines,
        capacity: state.capacity.as_ref(),
        overflow: overflow_view.as_ref(),
        sizing,
        admin: context.role == Role::Admin,
    });
    let waiting: Vec<_> = waiting_jobs
        .into_iter()
        .map(|job| job_views::job(job, context.at))
        .collect();
    let running: Vec<_> = jobs
        .into_iter()
        .filter(|job| job.state == JobState::Running)
        .map(|job| job_views::job(job, context.at))
        .collect();
    let overflow = match overflow_view {
        Some(view) => {
            let counted = view.spend.counted_usd().ok();
            let next = context
                .at
                .date_naive()
                .with_day(1)
                .and_then(|date| date.checked_add_months(Months::new(1)))
                .ok_or(Error::Unavailable)?
                .and_time(NaiveTime::MIN)
                .and_utc();
            Some(JobOverflow {
                connected: view.account.is_some(),
                key_rejected: view
                    .account
                    .as_ref()
                    .is_some_and(|account| account.rejected_at.is_some()),
                billing_required: view.state.billing_required,
                retry_at: view.state.retry_at,
                limit_cents: view.limit_usd.map(billing_views::cents),
                counted_cents: counted.map(billing_views::cents),
                limit_reached: counted
                    .zip(view.limit_usd)
                    .map(|(counted, limit)| counted >= limit),
                measured_complete: view.spend.complete(),
                next_month: next,
                refused_sizes: view.state.refused_sizes,
            })
        }
        None => None,
    };
    Ok(JobsLivePage {
        waiting_labels,
        waiting_count: complete_waiting.then(|| waiting.len().to_string()),
        on_boat: running
            .iter()
            .filter(|job| {
                matches!(
                    job.location,
                    Some(crate::wire::jobs::JobLocation::Boat { .. })
                )
            })
            .count()
            .to_string(),
        running,
        waiting,
        own_slots_used: used.map(|value| value.to_string()),
        own_slots_total: slots.map(|value| value.to_string()),
        warm_runners,
        starting_runners,
        visibility,
        overflow_enabled: settings.overflow_enabled,
        overflow_wait_seconds: match settings.overflow_wait {
            OverflowWait::AtOnce => 0,
            OverflowWait::Seconds15 => 15,
            OverflowWait::Seconds30 => 30,
            OverflowWait::Seconds60 => 60,
            OverflowWait::Seconds120 => 120,
        },
        overflow,
        can_manage: context.role == Role::Admin,
        banner: page_context::banner(&mut context)?,
    })
}
