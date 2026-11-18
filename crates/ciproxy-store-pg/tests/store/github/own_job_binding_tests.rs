//! Exact immutable Worker job bindings persist through runner shutdown.

use ciproxy_protocol::identifiers::JobId;
use ciproxy_store_interface::job_bindings::{OwnJobBinding, WorkerJobId};
use ciproxy_store_interface::own_runners::{OwnRunner, OwnRunnerObservation};
use ciproxy_store_interface::runners::{RunnerPlace, RunnerState};
use ciproxy_store_interface::store::StoreFactory;
use ciproxy_store_pg::factory::PgStoreFactory;
use uuid::Uuid;

use super::super::support::{Database, at, machine, runner};

struct Fixture {
    database: Database,
    workspace_id: Uuid,
    runner: OwnRunner,
    binding: OwnJobBinding,
}

fn fixture() -> Fixture {
    let database = Database::new();
    let workspace = database.workspace();
    let organization = database.organization(&workspace);
    let (label, scale_set) = database.label_and_scale_set(&organization);
    let machine = machine();
    database
        .store
        .create_machine(workspace.id, &machine)
        .unwrap();
    let runner = OwnRunner {
        worker_job_id: None,
        registration_scale_set_id: Some(scale_set.github_id),
        requested_size: label.size,
        slot_indices: vec![0],
        runner: runner(
            &label,
            &scale_set,
            RunnerPlace::Machine {
                machine_id: machine.id,
                first_slot: 0,
            },
        ),
        label_name: label.name,
        command_id: Uuid::new_v4(),
        issued_at: at(),
        acknowledged: false,
        confirmed_at: None,
    };
    database
        .store
        .create_own_runner(workspace.id, &runner)
        .unwrap();
    let binding = OwnJobBinding {
        runner_id: runner.runner.id,
        organization_id: organization.id,
        machine_id: machine.id,
        github_runner_id: 84,
        worker_job_id: WorkerJobId(JobId("worker-job-41".to_owned())),
    };
    Fixture {
        database,
        workspace_id: workspace.id,
        runner,
        binding,
    }
}

#[test]
fn own_worker_job_binding_is_immutable_and_survives_shutdown_and_restart() {
    let fixture = fixture();
    let store = &fixture.database.store;
    let workspace = fixture.workspace_id;
    let binding = &fixture.binding;
    assert!(store.bind_own_job(workspace, binding).unwrap());
    assert!(store.bind_own_job(workspace, binding).unwrap());
    let mut conflict = binding.clone();
    conflict.worker_job_id = WorkerJobId(JobId("worker-job-42".to_owned()));
    assert!(!store.bind_own_job(workspace, &conflict).unwrap());
    let mut expected = fixture.runner.clone();
    expected.worker_job_id = Some(binding.worker_job_id.clone());
    assert_eq!(
        store.own_runners(workspace).unwrap(),
        vec![expected.clone()]
    );

    let stop_command = Uuid::new_v4();
    store
        .retire_own_runner(
            workspace,
            binding.runner_id,
            stop_command,
            at(),
            Some(binding.github_runner_id),
        )
        .unwrap();
    expected.runner.state = RunnerState::Stopping;
    expected.command_id = stop_command;
    assert_eq!(
        store.own_runners(workspace).unwrap(),
        vec![expected.clone()]
    );
    store
        .record_own_command_result(workspace, stop_command, true, at())
        .unwrap();
    store
        .observe_own_runner(
            workspace,
            binding.organization_id,
            binding.runner_id,
            OwnRunnerObservation {
                state: RunnerState::Stopped,
                github_id: Some(binding.github_runner_id),
                observed_at: at(),
            },
        )
        .unwrap();
    expected.runner.state = RunnerState::Stopped;
    expected.confirmed_at = Some(at());
    expected.acknowledged = true;
    assert!(!store.bind_own_job(workspace, binding).unwrap());
    assert_eq!(
        store.own_runners(workspace).unwrap(),
        vec![expected.clone()]
    );

    let restarted = PgStoreFactory.connect(&fixture.database.url).unwrap();
    assert_eq!(restarted.own_runners(workspace).unwrap(), vec![expected]);
}

#[test]
fn own_worker_job_binding_rejects_inexact_or_closed_runner_scope() {
    let fixture = fixture();
    let store = &fixture.database.store;
    let workspace = fixture.workspace_id;
    let binding = &fixture.binding;
    assert!(store.bind_own_job(workspace, binding).unwrap());
    let other_workspace = fixture.database.workspace();
    let saved_workspace = store.workspace(workspace).unwrap().unwrap();
    let other_organization = fixture.database.organization(&saved_workspace);
    let other_machine = machine();
    store.create_machine(workspace, &other_machine).unwrap();
    let mut other = fixture.runner.clone();
    other.runner.id = Uuid::new_v4();
    other.runner.github_id = Some(85);
    other.command_id = Uuid::new_v4();
    store.create_own_runner(workspace, &other).unwrap();
    let other_runner_id = other.runner.id;
    let mut unregistered = other.clone();
    unregistered.runner.id = Uuid::new_v4();
    unregistered.runner.github_id = None;
    unregistered.command_id = Uuid::new_v4();
    store.create_own_runner(workspace, &unregistered).unwrap();
    let mut stopped = other;
    stopped.runner.id = Uuid::new_v4();
    stopped.runner.github_id = Some(84);
    stopped.runner.state = RunnerState::Stopped;
    stopped.command_id = Uuid::new_v4();
    store.create_own_runner(workspace, &stopped).unwrap();
    let mut no_command = fixture.runner.runner.clone();
    no_command.id = Uuid::new_v4();
    store.create_runner(workspace, &no_command).unwrap();

    let before = store.own_runners(workspace).unwrap();
    let cases = [
        ("workspace", other_workspace.id, binding.clone()),
        (
            "organization",
            workspace,
            OwnJobBinding {
                organization_id: other_organization.id,
                ..binding.clone()
            },
        ),
        (
            "machine",
            workspace,
            OwnJobBinding {
                machine_id: other_machine.id,
                ..binding.clone()
            },
        ),
        (
            "unknown runner",
            workspace,
            OwnJobBinding {
                runner_id: Uuid::new_v4(),
                ..binding.clone()
            },
        ),
        (
            "other runner",
            workspace,
            OwnJobBinding {
                runner_id: other_runner_id,
                ..binding.clone()
            },
        ),
        (
            "GitHub runner",
            workspace,
            OwnJobBinding {
                github_runner_id: 85,
                ..binding.clone()
            },
        ),
        (
            "unregistered runner",
            workspace,
            OwnJobBinding {
                runner_id: unregistered.runner.id,
                ..binding.clone()
            },
        ),
        (
            "stopped runner",
            workspace,
            OwnJobBinding {
                runner_id: stopped.runner.id,
                ..binding.clone()
            },
        ),
        (
            "missing command",
            workspace,
            OwnJobBinding {
                runner_id: no_command.id,
                ..binding.clone()
            },
        ),
    ];
    for (scope, workspace_id, attempted) in cases {
        assert!(
            !store.bind_own_job(workspace_id, &attempted).unwrap(),
            "scope: {scope}"
        );
        assert_eq!(
            store.own_runners(workspace).unwrap(),
            before,
            "scope: {scope}"
        );
    }
}
