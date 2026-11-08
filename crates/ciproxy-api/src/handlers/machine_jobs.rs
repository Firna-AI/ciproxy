//! Exact report-job identity joins to the shared canonical dashboard projection.

use chrono::{DateTime, Utc};
use ciproxy_protocol::reports::{SlotOccupancy, SlotReport};
use ciproxy_protocol::resources::Slots;
use ciproxy_store_interface::job_views::DashboardJob;
use uuid::Uuid;

use crate::wire::jobs::JobView;

use super::job_views;

pub(super) fn resolve(
    jobs: &[DashboardJob],
    machine: Uuid,
    slot: &SlotReport,
    at: DateTime<Utc>,
) -> Option<JobView> {
    let SlotOccupancy::Job { runner_id, job_id } = &slot.occupancy else {
        return None;
    };
    let runner = Uuid::parse_str(&runner_id.0).ok()?;
    let reported = Uuid::parse_str(&job_id.0).ok()?;
    let mut matching = jobs.iter().filter(|job| {
        job.runner_id == Some(runner)
            && job.machine_id == Some(machine)
            && job
                .first_slot
                .zip(job.actual_size)
                .is_some_and(|(first, size)| {
                    u64::try_from(first).ok().is_some_and(|first| {
                        slot.index >= first
                            && slot.index < first.saturating_add(Slots::for_vm(size).0)
                    })
                })
            && (job.id == reported || job.reported_ids.contains(&reported))
    });
    let job = matching.next()?;
    if matching.next().is_some() {
        return None;
    }
    Some(job_views::job(job.clone(), at))
}
