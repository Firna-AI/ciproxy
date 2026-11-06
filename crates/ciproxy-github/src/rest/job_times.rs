//! Real job timing from every run-job page and execution attempt.

use chrono::{DateTime, Utc};
use ciproxy_store_interface::github_timing::JobTimes;
use reqwest::Method;
use serde::Deserialize;

use crate::error::{Result, ResultExt};

use super::http::RestClient;

#[derive(Deserialize)]
struct Page {
    total_count: usize,
    jobs: Vec<Job>,
}
#[derive(Deserialize)]
struct Job {
    id: i64,
    run_id: i64,
    runner_id: Option<i64>,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    status: String,
}

impl RestClient {
    pub(super) async fn times(
        &self,
        installation_id: i64,
        repository: &str,
        run_id: i64,
        runner_id: i64,
    ) -> Result<Option<JobTimes>> {
        let mut page = 1usize;
        let mut seen = 0usize;
        let mut matched: Option<JobTimes> = None;
        loop {
            let path = format!(
                "repos/{repository}/actions/runs/{run_id}/jobs?filter=all&per_page=100&page={page}"
            );
            let response = Self::success(
                self.request(installation_id, Method::GET, &path, &())
                    .await?,
            )?;
            let batch: Page = response.json().await.internal()?;
            if batch.jobs.is_empty() {
                break;
            }
            seen += batch.jobs.len();
            for job in batch
                .jobs
                .into_iter()
                .filter(|job| job.run_id == run_id && job.runner_id == Some(runner_id))
            {
                if matched
                    .as_ref()
                    .is_some_and(|old| old.actions_job_id != job.id)
                {
                    return Ok(None);
                }
                matched = Some(JobTimes {
                    actions_job_id: job.id,
                    started_at: job.started_at,
                    completed_at: job.completed_at,
                    terminal: job.status == "completed",
                });
            }
            if seen >= batch.total_count {
                break;
            }
            page += 1;
        }
        Ok(matched)
    }
}
