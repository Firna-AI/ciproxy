//! Preserve each accepted Boat delivery in the common scaler snapshot.

use ciproxy_protocol::identifiers::{JobId, LabelName, OrganizationId, RunnerId};
use ciproxy_protocol::resources::{GIGABYTE, VmSize};
use ciproxy_scaler::boat::{BoatRunner, Sandbox, SandboxId};
use ciproxy_scaler::snapshot::{Runner, RunnerState, Snapshot};
use ciproxy_store_interface::boat_runtime::BoatRunnerIntent;
use ciproxy_store_interface::github::{DemandPhase, GithubJob};
use ciproxy_store_interface::labels::Label;
use ciproxy_store_interface::overflow::record::{StartPhase, StartRecord};

pub(crate) fn append(
    snapshot: &mut Snapshot,
    intents: &[BoatRunnerIntent],
    records: &[StartRecord],
    jobs: &[GithubJob],
    labels: &[Label],
) {
    for intent in intents {
        let record = records
            .iter()
            .find(|record| record.delivery_id == intent.delivery_id);
        if record.is_some_and(|record| record.ends_at.is_some()) {
            continue;
        }
        let running = jobs.iter().find(|job| {
            job.organization_id == intent.organization_id
                && job.scale_set_id == intent.scale_set_id
                && job.phase == DemandPhase::Started
                && job.runner_github_id.is_some()
                && job.runner_github_id == intent.github_id
        });
        let completed = jobs.iter().any(|job| {
            job.organization_id == intent.organization_id
                && job.scale_set_id == intent.scale_set_id
                && job.phase == DemandPhase::Completed
                && job.runner_github_id.is_some()
                && job.runner_github_id == intent.github_id
        });
        if running.is_none() && !labels.iter().any(|label| label.id == intent.label_id) {
            continue;
        }
        let state = if let Some(job) = running {
            RunnerState::Running(JobId(job.id.to_string()))
        } else if completed {
            RunnerState::Exited
        } else {
            match record.map(|record| record.phase) {
                Some(StartPhase::Running) => RunnerState::Ready,
                Some(StartPhase::Deleting) => RunnerState::Retiring,
                _ => RunnerState::Starting,
            }
        };
        let size =
            record
                .and_then(|record| record.dispatch.as_ref())
                .map_or(intent.size, |parameters| VmSize {
                    vcpu: parameters.size.vcpus,
                    memory_bytes: parameters.size.memory_bytes,
                    disk_bytes: parameters.size.disk_bytes,
                });
        let requested_size = record.map_or(intent.requested_size, |record| VmSize {
            vcpu: record.requested_vcpus,
            memory_bytes: u64::from(record.requested_memory_gb) * GIGABYTE,
            disk_bytes: u64::from(record.requested_disk_gb) * GIGABYTE,
        });
        snapshot.sandboxes.push(Sandbox {
            id: SandboxId(
                record
                    .and_then(|record| record.sandbox.as_ref())
                    .map_or_else(
                        || intent.runner_id.to_string(),
                        |sandbox| sandbox.id().to_owned(),
                    ),
            ),
            marked: true,
            runner: Some(BoatRunner {
                required_root_disk_bytes: record
                    .map_or(intent.required_root_disk_bytes, |record| {
                        record.required_root_disk_bytes
                    }),
                runner: Runner {
                    id: RunnerId(intent.runner_id.to_string()),
                    organization_id: OrganizationId(intent.organization_id.to_string()),
                    label: LabelName(intent.label_name.clone()),
                    requested_size,
                    size,
                    state,
                },
            }),
        });
    }
}
