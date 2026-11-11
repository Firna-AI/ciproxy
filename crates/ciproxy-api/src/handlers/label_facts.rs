//! Shared page facts from real overflow observations and canonical taken jobs.

use chrono::{DateTime, Duration, Utc};
use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_overflow::view::OverflowView;
use ciproxy_store_interface::job_views::DashboardJob;
use uuid::Uuid;

use crate::error::Result;
use crate::pages::PageServices;
use crate::state::{ApiState, query};
use crate::wire::settings::{LabelFit, LabelInput, LabelView};

use super::label_boat;

pub(super) struct Facts {
    overflow: Option<OverflowView>,
    sizing: RootSizing,
    jobs: Vec<DashboardJob>,
    at: DateTime<Utc>,
}

impl Facts {
    pub(super) async fn read(
        state: &ApiState,
        workspace: Uuid,
        pages: Option<&PageServices>,
    ) -> Result<Self> {
        let overflow = pages
            .and_then(|pages| pages.overflow.as_ref())
            .or(state.overflow.as_ref());
        let overflow = match overflow {
            Some(service) => service.view(workspace).await.ok(),
            None => None,
        };
        let sizing = pages
            .and_then(|pages| pages.boat_root_sizing)
            .unwrap_or(RootSizing {
                image_root_bytes: state.config.image_disk_bytes,
            });
        let at = state.runtime.now();
        let jobs = query(state.store.clone(), move |store| {
            store.dashboard_jobs(workspace, at - Duration::days(30))
        })
        .await?;
        Ok(Self {
            overflow,
            sizing,
            jobs,
            at,
        })
    }

    pub(super) fn enrich(&self, view: &mut LabelView) {
        label_boat::enrich(
            &mut view.fit,
            &view.label,
            self.overflow.as_ref(),
            self.sizing,
        );
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

    pub(super) fn preview(&self, input: &LabelInput, fit: &mut LabelFit) {
        label_boat::enrich(fit, input, self.overflow.as_ref(), self.sizing);
    }
}
