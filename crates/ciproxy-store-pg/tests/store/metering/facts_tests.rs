//! Workspace-scoped facts and exact cost-history persistence.

use std::sync::Arc;

use chrono::Duration;
use ciproxy_metering::service::{Metering, StoredMetering};
use ciproxy_metering::usage::PaymentNotice;
use ciproxy_store_interface::jobs::{JobResult, JobState};
use ciproxy_store_interface::metering::facts::MachineRegistration;
use ciproxy_store_interface::metering::history::{BoatCost, MachineCost, Period, PlanCost};
use ciproxy_store_interface::runners::RunnerPlace;
use uuid::Uuid;

use super::super::support::{Database, at, job, machine, runner};
use super::support::{free, time, usd};

#[test]
fn metering_reads_all_workspace_organizations_and_current_machines() {
    let database = Database::new();
    let workspace = database.workspace();
    let other = database.workspace();
    let org = database.organization(&workspace);
    let second = database.organization(&workspace);
    let foreign = database.organization(&other);
    let mut own = machine();
    own.draining = true;
    own.kvm_available = false;
    own.last_contact_at = None;
    database.store.create_machine(workspace.id, &own).unwrap();
    database.store.create_machine(other.id, &machine()).unwrap();
    let removed = machine();
    database
        .store
        .create_machine(workspace.id, &removed)
        .unwrap();
    database
        .store
        .record_machine_registration(workspace.id, removed.id, time("2026-11-01T00:00:00Z"))
        .unwrap();
    database
        .store
        .remove_machine(workspace.id, removed.id, at())
        .unwrap();
    for organization in [&org, &second, &foreign] {
        let (label, scale) = database.label_and_scale_set(organization);
        let vm = runner(
            &label,
            &scale,
            RunnerPlace::Boat {
                key_id: None,
                sandbox_id: Uuid::new_v4().to_string(),
                size_name: "actual".to_owned(),
            },
        );
        database
            .store
            .create_runner(organization.workspace_id, &vm)
            .unwrap();
        database
            .store
            .save_boat_cost(
                organization.workspace_id,
                &BoatCost {
                    runner_id: vm.id,
                    starts_at: at(),
                    ends_at: Some(at() + Duration::minutes(10)),
                    start_usd: usd("0.1"),
                    minute_usd: usd("0.02"),
                },
            )
            .unwrap();
        let mut taken = job(&label, &scale);
        taken.runner_id = Some(vm.id);
        taken.started_at = Some(at());
        taken.state = JobState::Completed;
        taken.completed_at = Some(at() + Duration::minutes(5));
        taken.result = Some(JobResult::Failure);
        assert!(
            database
                .store
                .insert_job(organization.workspace_id, &taken)
                .unwrap()
        );
        assert!(
            !database
                .store
                .insert_job(organization.workspace_id, &taken)
                .unwrap()
        );
        let mut cancelled = job(&label, &scale);
        cancelled.github_id = 43;
        cancelled.state = JobState::Completed;
        cancelled.result = Some(JobResult::Cancelled);
        database
            .store
            .insert_job(organization.workspace_id, &cancelled)
            .unwrap();
    }
    let service: Arc<dyn Metering> = Arc::new(StoredMetering::new(database.store.clone()));
    let use_now = service
        .usage(
            workspace.id,
            Period {
                starts_at: time("2026-11-04T12:00:00Z"),
                ends_at: time("2026-12-05T12:00:00Z"),
            },
            free(),
            at() + Duration::hours(1),
            PaymentNotice::None,
        )
        .unwrap();
    assert_eq!(
        (
            use_now.starts,
            use_now.machines,
            use_now.organizations,
            use_now.organizations_over
        ),
        (2, 1, 2, 1)
    );
    database
        .store
        .record_workspace_start(workspace.id, at())
        .unwrap();
    database
        .store
        .record_machine_registration(workspace.id, own.id, at())
        .unwrap();
    let cost = MachineCost {
        id: Uuid::new_v4(),
        machine_id: own.id,
        starts_at: at(),
        ends_at: None,
        monthly_usd: usd("119.25"),
    };
    database
        .store
        .save_machine_cost(workspace.id, &cost)
        .unwrap();
    database
        .store
        .save_plan_cost(
            workspace.id,
            &PlanCost {
                id: Uuid::new_v4(),
                starts_at: at(),
                ends_at: None,
                monthly_usd: usd("49"),
            },
        )
        .unwrap();
    service
        .refresh(workspace.id, at() + Duration::hours(1))
        .unwrap();
    service
        .refresh(other.id, at() + Duration::hours(1))
        .unwrap();
    let snapshot = database
        .store
        .savings_snapshot(
            workspace.id,
            at().date_naive(),
            (at() + Duration::days(1)).date_naive(),
        )
        .unwrap();
    let history = snapshot.costs;
    assert_eq!(
        (
            history.workspace_started_at,
            history.machines,
            history.plans[0].monthly_usd
        ),
        (Some(at()), vec![cost.clone()], usd("49"))
    );
    assert!(matches!(
        database.store.save_machine_cost(other.id, &cost),
        Err(ciproxy_store_interface::error::Error::NotFound)
    ));
    assert_eq!(
        database
            .store
            .savings_snapshot(
                other.id,
                at().date_naive(),
                (at() + Duration::days(1)).date_naive()
            )
            .unwrap()
            .costs
            .machines,
        vec![]
    );
    let mut registrations = snapshot.machines;
    registrations.sort_by_key(|machine| machine.machine_id);
    let mut expected = vec![
        MachineRegistration {
            machine_id: own.id,
            registered_at: Some(at()),
            removed_at: None,
        },
        MachineRegistration {
            machine_id: removed.id,
            registered_at: Some(time("2026-11-01T00:00:00Z")),
            removed_at: Some(at()),
        },
    ];
    expected.sort_by_key(|machine| machine.machine_id);
    assert_eq!(registrations, expected);
    assert_eq!(snapshot.days.len(), 1);
    assert_eq!(snapshot.days[0].sizes.len(), 1);
    assert_eq!(
        (
            snapshot.days[0].day,
            snapshot.days[0].sizes[0].boat.starts,
            snapshot.days[0].sizes[0].boat.minutes,
            snapshot.days[0].boat_usd
        ),
        (at().date_naive(), 2, usd("10"), usd("0.6"))
    );
    let foreign = database
        .store
        .savings_snapshot(
            other.id,
            at().date_naive(),
            (at() + Duration::days(1)).date_naive(),
        )
        .unwrap();
    assert_eq!(
        (
            foreign.machines.len(),
            foreign.costs.workspace_started_at,
            foreign.costs.plans.len()
        ),
        (1, None, 0)
    );
    assert_eq!(
        (
            foreign.days[0].sizes[0].boat.starts,
            foreign.days[0].sizes[0].boat.minutes,
            foreign.days[0].boat_usd
        ),
        (1, usd("5"), usd("0.3"))
    );
    for (from, until) in [
        (at() - Duration::days(1), at()),
        (at() + Duration::days(1), at() + Duration::days(2)),
    ] {
        assert_eq!(
            database
                .store
                .savings_snapshot(workspace.id, from.date_naive(), until.date_naive())
                .unwrap()
                .days,
            vec![]
        );
    }
}
