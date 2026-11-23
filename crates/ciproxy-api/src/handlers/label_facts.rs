//! Shared page facts from provider observations and canonical taken jobs.

use std::collections::BTreeMap;

use chrono::{DateTime, Duration, Utc};
use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_overflow::view::OverflowView;
use ciproxy_store_interface::job_views::DashboardJob;
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use uuid::Uuid;

use crate::error::Result;
use crate::pages::PageServices;
use crate::state::{ApiState, query};
use crate::wire::settings::LabelView;

use super::{label_provider::Observation, overflow_views};

pub(super) struct Facts<'a> {
    overflow: Option<OverflowView>,
    roots: Option<&'a BTreeMap<OverflowProvider, RootSizing>>,
    image_root_bytes: u64,
    enabled: bool,
    jobs: Vec<DashboardJob>,
    at: DateTime<Utc>,
}

impl<'a> Facts<'a> {
    pub(super) async fn read(
        state: &ApiState,
        workspace: Uuid,
        pages: Option<&'a PageServices>,
    ) -> Result<Self> {
        let overflow = overflow_views::observe(state, pages, workspace).await;
        let at = state.runtime.now();
        let (jobs, settings) = query(state.store.clone(), move |store| {
            Ok((
                store.dashboard_jobs(workspace, at - Duration::days(30))?,
                store.settings(workspace)?,
            ))
        })
        .await?;
        Ok(Self {
            overflow,
            roots: pages.map(|pages| &pages.provider_roots),
            image_root_bytes: state.config.image_disk_bytes,
            enabled: settings.overflow_enabled,
            jobs,
            at,
        })
    }

    pub(super) fn observation(&self) -> Observation<'_> {
        Observation {
            view: self.overflow.as_ref(),
            roots: self.roots,
            image_root_bytes: self.image_root_bytes,
            enabled: self.enabled,
            at: self.at,
        }
    }

    pub(super) fn enrich(&self, view: &mut LabelView) {
        let id = Uuid::parse_str(&view.id).ok();
        let jobs = self
            .jobs
            .iter()
            .filter(|job| Some(job.label_id) == id && job.runner_taken)
            .collect::<Vec<_>>();
        let since = self.at - Duration::days(30);
        view.jobs_last_30_days = (!jobs.iter().any(|job| job.started_at.is_none())).then(|| {
            jobs.iter()
                .filter(|job| {
                    job.started_at
                        .is_some_and(|at| since <= at && at <= self.at)
                })
                .count()
                .to_string()
        });
        view.job_count_since = Some(since);
        view.job_count_until = Some(self.at);
    }
}
