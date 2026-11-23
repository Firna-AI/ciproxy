//! Translate durable tenant data and current machine reports for the pure scaler.

use std::time::Duration;

use chrono::{DateTime, Utc};
use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_protocol::identifiers::{
    JobId, LabelName, MachineId, OrganizationId, RunnerId, WorkspaceId,
};
use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::resources::{GIGABYTE, Resources, VmSize};
use ciproxy_scaler::boat::{BoatAccount, MonthlySpend};
use ciproxy_scaler::registration::RegistrationTarget;
use ciproxy_scaler::snapshot::{
    Destinations, Label, Machine, NothingFits, Runner, RunnerState, Snapshot, WaitingJob,
    warm_label,
};
use ciproxy_store_interface::gateway::{MachineSnapshot, MachineState};
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::labels::{LabelPlace, WhenNothingFits};
use ciproxy_store_interface::organizations::AppState;
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use ciproxy_store_interface::own_runners::RoutingSnapshot;
use ciproxy_store_interface::runners::{self, RunnerPlace};
use ciproxy_store_interface::settings::OverflowWait;
use uuid::Uuid;

use crate::boat_snapshot;
use crate::warm::allocate;

pub(crate) fn build(
    input: &RoutingSnapshot,
    runnable: &[Uuid],
    now: DateTime<Utc>,
    root_sizing: RootSizing,
) -> Snapshot {
    let active: Vec<_> = input
        .organizations
        .iter()
        .filter(|o| o.app_state == AppState::Active && runnable.contains(&o.id))
        .collect();
    let history = |platform: Platform| -> Vec<(OrganizationId, u64)> {
        active
            .iter()
            .map(|organization| {
                let jobs = input
                    .jobs
                    .iter()
                    .filter(|job| {
                        job.organization_id == organization.id
                            && input.labels.iter().any(|label| {
                                label.id == job.label_id
                                    && label.name == warm_label(platform)
                                    && label.platform == platform
                            })
                            && job.assigned_at.is_some_and(|at| {
                                at <= now && at > now - chrono::Duration::hours(24)
                            })
                    })
                    .count() as u64;
                (OrganizationId(organization.id.to_string()), jobs)
            })
            .collect()
    };
    Snapshot {
        workspace_id: WorkspaceId(input.workspace_id.to_string()),
        root_sizing,
        now: instant(now),
        control_plane_available: true,
        organizations: active
            .iter()
            .map(|o| OrganizationId(o.id.to_string()))
            .collect(),
        labels: input
            .labels
            .iter()
            .map(|label| Label {
                targets: input
                    .scale_sets
                    .iter()
                    .filter(|set| set.label_id == label.id)
                    .map(|set| {
                        registration_target(
                            set.organization_id,
                            set.label_id,
                            set.id,
                            set.github_id,
                        )
                    })
                    .collect(),
                name: LabelName(label.name.clone()),
                platform: label.platform,
                size: size(label.size),
                destinations: Destinations {
                    own_machines: label.order.contains(&LabelPlace::Machine),
                    boat: label
                        .order
                        .contains(&LabelPlace::Provider(OverflowProvider::Boat)),
                },
                nothing_fits: match label.when_nothing_fits {
                    WhenNothingFits::Wait => NothingFits::Wait,
                    WhenNothingFits::LargestOverflow => NothingFits::LargestBoatSize,
                },
            })
            .collect(),
        queue: input
            .jobs
            .iter()
            .filter(|job| job.phase <= DemandPhase::Assigned)
            .filter(|job| runnable.contains(&job.organization_id))
            .filter_map(|job| {
                let label = input.labels.iter().find(|label| label.id == job.label_id)?;
                let set = input.scale_sets.iter().find(|set| {
                    set.id == job.scale_set_id
                        && set.label_id == label.id
                        && set.organization_id == job.organization_id
                })?;
                Some(WaitingJob {
                    target: registration_target(
                        set.organization_id,
                        set.label_id,
                        set.id,
                        set.github_id,
                    ),
                    queued_at: instant(job.assigned_at?),
                })
            })
            .collect(),
        machines: input
            .machines
            .iter()
            .map(|machine| machine_input(machine, input, now, &history(machine.machine.platform)))
            .collect(),
        boat: BoatAccount {
            enabled: input.settings.overflow_enabled,
            connected: false,
            spend: MonthlySpend { spent: 0, limit: 0 },
            sizes: Vec::new(),
            largest_size: None,
            refusals: Vec::new(),
        },
        sandboxes: boat_snapshot::project(input, &input.overflow.starts),
        overflow_wait: Duration::from_secs(match input.settings.overflow_wait {
            OverflowWait::AtOnce => 0,
            OverflowWait::Seconds15 => 15,
            OverflowWait::Seconds30 => 30,
            OverflowWait::Seconds60 => 60,
            OverflowWait::Seconds120 => 120,
        }),
    }
}

fn machine_input(
    machine: &MachineSnapshot,
    input: &RoutingSnapshot,
    now: DateTime<Utc>,
    history: &[(OrganizationId, u64)],
) -> Machine {
    let resources = machine
        .report
        .as_ref()
        .map(|report| Resources {
            slots: report.report.capacity,
            job_disk_bytes: report.report.job_disk_bytes,
        })
        .unwrap_or_default();
    let count = usize::try_from(machine.machine.warm_runners).unwrap_or(0);
    Machine {
        id: MachineId(machine.machine.id.to_string()),
        platform: machine.machine.platform,
        selected_image: machine.report.as_ref().and_then(|report| report.report.images.selected().cloned()),
        connected: machine.state(now) == MachineState::Ready,
        draining: machine.machine.draining,
        capacity: resources.slots, disk_budget_bytes: resources.job_disk_bytes,
        // A Mac has no Docker disks, as docker-disks.md says, so it leases no bytes.
        docker_leased_bytes: if machine.machine.platform == Platform::MacosArm64 { Some(0) } else { machine.report.as_ref().and_then(|report| report.report.docker_disks.as_ref().measured().map(|docker| docker.leased_bytes)) },
        warm_count: count, warm_allocation: allocate(count, history),
        runners: input.runners.iter().filter(|intent| intent.runner.state != runners::RunnerState::Stopped && matches!(intent.runner.place, RunnerPlace::Machine { machine_id, .. } if machine_id == machine.machine.id)).map(|intent| {
            let running_job = input.jobs.iter().find(|job| job.phase == DemandPhase::Started && job.runner_github_id == intent.runner.github_id && job.organization_id == intent.runner.organization_id && job.scale_set_id == intent.runner.scale_set_id);
            let state = if let Some(job) = running_job { RunnerState::Running(JobId(job.id.to_string())) } else { match intent.runner.state { runners::RunnerState::Starting => RunnerState::Starting, runners::RunnerState::Ready => RunnerState::Ready, runners::RunnerState::Busy => RunnerState::Running(JobId(intent.runner.id.to_string())), runners::RunnerState::Stopping => RunnerState::Retiring, runners::RunnerState::Stopped => RunnerState::Exited } };
            Runner { target: intent.registration_scale_set_id.or_else(|| (intent.runner.github_id.is_none() && intent.runner.state == runners::RunnerState::Starting).then(|| input.scale_sets.iter().find(|set| set.id == intent.runner.scale_set_id).map(|set| set.github_id)).flatten()).map(|github_id| registration_target(intent.runner.organization_id, intent.runner.label_id, intent.runner.scale_set_id, github_id)), id: RunnerId(intent.runner.id.to_string()), organization_id: OrganizationId(intent.runner.organization_id.to_string()), label: LabelName(intent.label_name.clone()), requested_size: size(intent.requested_size), size: intent.runner.size, state }
        }).collect(),
    }
}

pub(crate) fn registration_target(
    organization: Uuid,
    label: Uuid,
    binding: Uuid,
    github_id: i64,
) -> RegistrationTarget {
    RegistrationTarget {
        organization_id: OrganizationId(organization.to_string()),
        label_id: label.to_string(),
        scale_set_id: binding.to_string(),
        github_id,
    }
}

pub(crate) fn size(size: ciproxy_store_interface::vm_size::VmSize) -> VmSize {
    VmSize {
        vcpu: u32::try_from(size.vcpus).unwrap_or(0),
        memory_bytes: u64::try_from(size.memory_gb).unwrap_or(0) * GIGABYTE,
        disk_bytes: u64::try_from(size.disk_gb).unwrap_or(0) * GIGABYTE,
    }
}

/// Convert a UTC timestamp to the scaler snapshot clock.
pub(crate) fn instant(at: DateTime<Utc>) -> Duration {
    at.signed_duration_since(DateTime::UNIX_EPOCH)
        .to_std()
        .unwrap_or(Duration::ZERO)
}

#[cfg(test)]
#[path = "_tests_/snapshot/mod.rs"]
pub(crate) mod tests;
