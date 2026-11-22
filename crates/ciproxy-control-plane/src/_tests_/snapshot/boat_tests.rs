//! Saved Boat supply covers demand before any optional provider refresh.

use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_protocol::identifiers::JobId;
use ciproxy_protocol::resources::VmSize;
use ciproxy_scaler::actions::Action;
use ciproxy_scaler::decision::decide;
use ciproxy_scaler::snapshot::RunnerState;
use ciproxy_store_interface::boat_runtime::BoatRunnerIntent;
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::overflow::record::{StartPhase, StartRecord};
use ciproxy_store_interface::own_runners::RoutingSnapshot;
use ciproxy_store_interface::runners;
use uuid::Uuid;

use crate::snapshot::build;

use super::fixtures::{at, input, job};

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
    let intent = input.boat_intents[0].clone();
    input.boat_runners.push(runners::Runner {
        id: intent.runner_id,
        organization_id: intent.organization_id,
        label_id: intent.label_id,
        scale_set_id: intent.scale_set_id,
        github_id: intent.github_id,
        state: runners::RunnerState::Busy,
        size: intent.size,
        place: runners::RunnerPlace::Boat {
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

fn accepted() -> RoutingSnapshot {
    let mut input = input();
    input.labels[0].allow_boat = true;
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
    input.boat_intents.push(BoatRunnerIntent {
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
