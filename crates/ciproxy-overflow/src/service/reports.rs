//! Best-effort final usage and bounded live Boat sample reports.

use chrono::{DateTime, TimeDelta, Utc};
use ciproxy_protocol::secret::Secret;
use ciproxy_protocol::usage::JobUsage;
use ciproxy_store_interface::job_usage::BoatSampleWrite;
use ciproxy_store_interface::overflow::record::StartRecord;
use tokio::task::spawn_blocking;
use uuid::Uuid;

use crate::error::{Error, Result, ResultExt};
use crate::job_usage::UsageLimits;
use crate::secrets::digest;

use super::OverflowService;

const SAMPLE_INTERVAL: TimeDelta = TimeDelta::seconds(10);

impl OverflowService {
    /// Save optional final usage after the durable exit claim.
    pub(super) async fn save_exit_usage(
        &self,
        workspace_id: Uuid,
        record: &StartRecord,
        received_at: DateTime<Utc>,
        usage: JobUsage,
    ) -> Result<()> {
        let Some(limits) = limits(record, received_at) else {
            tracing::warn!("dropped final Boat job usage: no saved sandbox size");
            return Ok(());
        };
        let runner_id = record.runner_id;
        let store = self.dependencies.store.clone();
        spawn_blocking(move || limits.save_final(store.as_ref(), workspace_id, runner_id, usage))
            .await
            .internal()?
            .internal()
    }

    /// Check the sandbox token, then save valid live samples within the rate limit.
    pub(super) async fn sample_report(
        &self,
        token: Secret,
        runner_id: Uuid,
        usage: JobUsage,
    ) -> Result<()> {
        let received_at = self.dependencies.clock.now();
        let store = self.dependencies.store.clone();
        let hash = digest(&token);
        spawn_blocking(move || {
            let target = store
                .boat_usage_target(&hash, runner_id)
                .internal()?
                .ok_or(Error::InvalidToken)?;
            let record = target.record;
            if record.reported_at.is_some() || record.ends_at.is_some() {
                return Err(Error::SamplesClosed);
            }
            let limits = limits(&record, received_at).ok_or(Error::SamplesClosed)?;
            let report = limits.prepare(usage, false).ok_or(Error::InvalidSamples)?;
            match store
                .save_boat_sample(
                    target.workspace_id,
                    record.delivery_id,
                    runner_id,
                    &report,
                    received_at - SAMPLE_INTERVAL,
                )
                .internal()?
            {
                BoatSampleWrite::Saved => Ok(()),
                BoatSampleWrite::Closed => Err(Error::SamplesClosed),
                BoatSampleWrite::TooSoon => Err(Error::SamplesTooSoon),
            }
        })
        .await
        .internal()?
    }
}

fn limits(record: &StartRecord, received_at: DateTime<Utc>) -> Option<UsageLimits> {
    record.sandbox.as_ref()?;
    let size = &record.dispatch.as_ref()?.size;
    Some(UsageLimits {
        vcpus: size.vcpus,
        memory_bytes: size.memory_bytes,
        started_at: record.starts_at,
        received_at,
    })
}
