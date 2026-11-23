//! Project provider accounts and accepted deliveries into the scaler snapshot.

use chrono::DateTime;
use ciproxy_overflow::error::Result;
use ciproxy_overflow::registry::ProviderRegistry;
use ciproxy_overflow::view::OverflowView;
use ciproxy_protocol::identifiers::{JobId, LabelName, OrganizationId, RunnerId};
use ciproxy_protocol::resources::{GIGABYTE, VmSize};
use ciproxy_scaler::provider::{ProviderAccount, ProviderRunner, Sandbox, SandboxId};
use ciproxy_scaler::snapshot::{Runner, RunnerState};
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use ciproxy_store_interface::overflow::record::{StartPhase, StartRecord};
use ciproxy_store_interface::own_runners::RoutingSnapshot;
use ciproxy_store_interface::runners;

use crate::snapshot::{instant, provider_id, registration_target};

pub(crate) fn accounts<'a>(
    view: &'a OverflowView,
    registry: &'a ProviderRegistry,
    refreshed: &'a [OverflowProvider],
) -> impl Iterator<Item = (OverflowProvider, Result<ProviderAccount>)> + 'a {
    view.providers.iter().map(|view| {
        let provider = view.provider;
        let configured = registry.get(provider).ok();
        let available = configured.is_some();
        let facts = if available && refreshed.contains(&provider) {
            view.scaler_account(DateTime::UNIX_EPOCH)
        } else {
            view.retained_scaler_account()
        };
        (
            provider,
            facts.map(|facts| ProviderAccount {
                id: provider_id(provider),
                available,
                root_sizing: configured
                    .map(|entry| entry.image.root_sizing)
                    .unwrap_or_default(),
                facts,
            }),
        )
    })
}

pub(crate) fn project(input: &RoutingSnapshot, records: &[StartRecord]) -> Vec<Sandbox> {
    let mut sandboxes = Vec::new();
    for intent in &input.overflow_intents {
        let record = records
            .iter()
            .find(|record| record.delivery_id == intent.delivery_id);
        if record.is_some_and(|record| record.ends_at.is_some()) {
            continue;
        }
        let running = input.jobs.iter().find(|job| {
            job.organization_id == intent.organization_id
                && job.scale_set_id == intent.scale_set_id
                && job.phase == DemandPhase::Started
                && job.runner_github_id.is_some()
                && job.runner_github_id == intent.github_id
        });
        let completed = input.jobs.iter().any(|job| {
            job.organization_id == intent.organization_id
                && job.scale_set_id == intent.scale_set_id
                && job.phase == DemandPhase::Completed
                && job.runner_github_id.is_some()
                && job.runner_github_id == intent.github_id
        });
        let saved = input.overflow_runners.iter().find(|runner| {
            runner.id == intent.runner_id
                && runner.organization_id == intent.organization_id
                && runner.scale_set_id == intent.scale_set_id
        });
        let state = if let Some(job) = running {
            RunnerState::Running(JobId(job.id.to_string()))
        } else if completed {
            RunnerState::Exited
        } else {
            match (
                saved.map(|runner| runner.state),
                record.map(|record| record.phase),
            ) {
                (Some(runners::RunnerState::Busy), _) => {
                    RunnerState::Running(JobId(intent.runner_id.to_string()))
                }
                (Some(runners::RunnerState::Stopped), _) => RunnerState::Exited,
                (Some(runners::RunnerState::Stopping), _) | (_, Some(StartPhase::Deleting)) => {
                    RunnerState::Retiring
                }
                (Some(runners::RunnerState::Ready), _)
                | (Some(runners::RunnerState::Starting) | None, Some(StartPhase::Running)) => {
                    RunnerState::Ready
                }
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
        sandboxes.push(Sandbox {
            provider: provider_id(intent.provider),
            id: SandboxId(
                record
                    .and_then(|record| record.sandbox.as_ref())
                    .map_or_else(
                        || intent.runner_id.to_string(),
                        |sandbox| sandbox.id().to_owned(),
                    ),
            ),
            marked: true,
            runner: Some(ProviderRunner {
                launched_at: record.and_then(|record| record.launched_at).map(instant),
                required_root_disk_bytes: record
                    .map_or(intent.required_root_disk_bytes, |record| {
                        record.required_root_disk_bytes
                    }),
                runner: Runner {
                    target: intent
                        .registration_scale_set_id
                        .or_else(|| {
                            intent
                                .github_id
                                .is_none()
                                .then(|| {
                                    input
                                        .scale_sets
                                        .iter()
                                        .find(|set| set.id == intent.scale_set_id)
                                        .map(|set| set.github_id)
                                })
                                .flatten()
                        })
                        .map(|github_id| {
                            registration_target(
                                intent.organization_id,
                                intent.label_id,
                                intent.scale_set_id,
                                github_id,
                            )
                        }),
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
    sandboxes
}
