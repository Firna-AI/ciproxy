//! Saved Boat supply covers demand before any optional provider refresh.

use std::sync::Arc;

use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_overflow::boat::BoatFactory;
use ciproxy_overflow::registry::ProviderRegistry;
use ciproxy_overflow::view::OverflowView;
use ciproxy_protocol::identifiers::JobId;
use ciproxy_protocol::resources::VmSize;
use ciproxy_scaler::actions::{Action, StopReason};
use ciproxy_scaler::decision::decide;
use ciproxy_scaler::provider::{ProviderId, SandboxId};
use ciproxy_scaler::snapshot::RunnerState;
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::labels::LabelPlace;
use ciproxy_store_interface::overflow::known::KnownSandbox;
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use ciproxy_store_interface::overflow::record::{StartPhase, StartRecord};
use ciproxy_store_interface::overflow_runtime::OverflowRunnerIntent;
use ciproxy_store_interface::own_runners::RoutingSnapshot;
use ciproxy_store_interface::runners;
use uuid::Uuid;

use crate::overflow_snapshot::{self, ProviderChecks};
use crate::snapshot::build;

use super::fixtures::{at, input, job};

#[test]
fn a_provider_without_a_start_image_has_no_start_geometry() {
    let view = OverflowView::from_snapshot(input().overflow, at()).unwrap();
    let registry = ProviderRegistry::default().with_provider(
        OverflowProvider::Boat,
        Arc::new(BoatFactory::new()),
        None,
    );

    let account = overflow_snapshot::accounts(&view, &registry, ProviderChecks::InProgress)
        .find(|(provider, _)| *provider == OverflowProvider::Boat)
        .unwrap()
        .1
        .unwrap();

    assert_eq!(account.start_geometry, None);
}

#[test]
fn accepted_boat_supply_covers_waiting_demand_before_account_refresh() {
    let input = accepted();
    let snapshot = build(
        &input,
        &[input.organizations[0].id],
        at(),
        RootSizing::default(),
    );
    assert_eq!(snapshot.sandboxes.len(), 1);
    assert_eq!(snapshot.queue.len(), 1);
    assert!(snapshot.machines[0].connected);
    assert_eq!(
        decide(&snapshot)
            .into_iter()
            .filter(|action| matches!(action, Action::StartRunner(_)))
            .collect::<Vec<_>>(),
        vec![]
    );
}

#[test]
fn boat_job_evidence_takes_priority_over_saved_busy_state() {
    let mut input = accepted();
    let intent = input.overflow_intents[0].clone();
    input.overflow_runners.push(runners::Runner {
        id: intent.runner_id,
        organization_id: intent.organization_id,
        label_id: intent.label_id,
        scale_set_id: intent.scale_set_id,
        github_id: intent.github_id,
        state: runners::RunnerState::Busy,
        size: intent.size,
        place: runners::RunnerPlace::Overflow {
            provider: OverflowProvider::Boat,
            key_id: None,
            sandbox_id: intent.runner_id.to_string(),
            size_name: "retained".to_owned(),
        },
    });
    let snapshot = build(
        &input,
        &[intent.organization_id],
        at(),
        RootSizing::default(),
    );
    assert_eq!(snapshot.sandboxes.len(), 1);
    assert_eq!(
        snapshot.sandboxes[0].runner.as_ref().unwrap().runner.state,
        RunnerState::Running(JobId(intent.runner_id.to_string()))
    );
    let mut completed = job(20, 2, 0);
    completed.scale_set_id = intent.scale_set_id;
    completed.runner_github_id = intent.github_id;
    input.jobs.push(completed);
    let snapshot = build(
        &input,
        &[intent.organization_id],
        at(),
        RootSizing::default(),
    );
    assert_eq!(
        snapshot.sandboxes[0].runner.as_ref().unwrap().runner.state,
        RunnerState::Exited
    );
    let mut started = job(21, 2, 0);
    started.scale_set_id = intent.scale_set_id;
    started.runner_github_id = intent.github_id;
    started.phase = DemandPhase::Started;
    input.jobs.push(started);
    let snapshot = build(
        &input,
        &[intent.organization_id],
        at(),
        RootSizing::default(),
    );
    assert_eq!(
        snapshot.sandboxes[0].runner.as_ref().unwrap().runner.state,
        RunnerState::Running(JobId(Uuid::from_u128(21).to_string()))
    );
}

#[test]
fn boat_ready_deadline_starts_at_launch_not_dispatch() {
    let mut input = accepted();
    let start = &mut input.overflow.starts[0];
    start.sandbox = Some(KnownSandbox::observed(
        "bx_23456789".to_owned(),
        at(),
        21600,
    ));
    start.launched_at = Some("2026-11-05T12:03:00Z".parse().unwrap());
    let deletions_at = |now: &str| {
        let snapshot = build(
            &input,
            &[input.organizations[0].id],
            now.parse().unwrap(),
            RootSizing::default(),
        );
        decide(&snapshot)
            .into_iter()
            .filter(|action| matches!(action, Action::DeleteSandbox { .. }))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        [
            deletions_at("2026-11-05T12:10:00Z"),
            deletions_at("2026-11-05T12:13:00Z"),
        ],
        [
            vec![],
            vec![Action::DeleteSandbox {
                provider: ProviderId::new("boat"),
                sandbox_id: SandboxId("bx_23456789".to_owned()),
                reason: StopReason::ReadyDeadlinePassed,
            }],
        ],
        "sandbox deletions at dispatch plus 10 minutes and launch plus 10 minutes"
    );
}

fn accepted() -> RoutingSnapshot {
    let mut input = input();
    input.labels[0]
        .order
        .push(LabelPlace::Provider(OverflowProvider::Boat));
    input.machines[0].machine.warm_runners = 0;
    let mut waiting = job(10, 2, 0);
    waiting.scale_set_id = input.scale_sets[0].id;
    waiting.phase = DemandPhase::Assigned;
    input.jobs.push(waiting);
    let requested_size = VmSize {
        vcpu: 2,
        memory_bytes: 8_000_000_000,
        disk_bytes: 80_000_000_000,
    };
    input.overflow_intents.push(OverflowRunnerIntent {
        provider: OverflowProvider::Boat,
        registration_scale_set_id: Some(input.scale_sets[0].github_id),
        delivery_id: Uuid::from_u128(11),
        runner_id: Uuid::from_u128(12),
        organization_id: input.organizations[0].id,
        label_id: input.labels[0].id,
        label_name: input.labels[0].name.clone(),
        scale_set_id: input.scale_sets[0].id,
        requested_size,
        size: VmSize {
            disk_bytes: 200_000_000_000,
            ..requested_size
        },
        required_root_disk_bytes: 160_000_000_000,
        github_id: Some(100),
        created_at: at(),
    });
    input.overflow.starts.push(StartRecord {
        provider: OverflowProvider::Boat,
        delivery_id: Uuid::from_u128(11),
        runner_id: Uuid::from_u128(12),
        organization_id: input.organizations[0].id,
        label_id: input.labels[0].id,
        scale_set_id: input.scale_sets[0].id,
        requested_vcpus: 2,
        requested_memory_gb: 8,
        requested_disk_gb: 80,
        image_root_bytes: 60_000_000_000,
        required_root_disk_bytes: 160_000_000_000,
        own_machine_fits: true,
        dispatch: None,
        key_id: Uuid::from_u128(13),
        deployment: "fixture".to_owned(),
        snapshot: "fixture".to_owned(),
        phase: StartPhase::Running,
        sandbox: None,
        starts_at: at(),
        launched_at: None,
        ends_at: None,
        billing: None,
        start_fee_bound_usd: None,
        start_usd: None,
        reported_at: None,
        runtime: Vec::new(),
        runtime_total: None,
        running_observed_at: None,
        runtime_complete: false,
    });
    input
}
