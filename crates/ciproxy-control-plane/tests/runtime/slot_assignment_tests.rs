//! Hosted reservations permit non-contiguous free slot sets.

use std::sync::{Arc, Mutex};

use ciproxy_control_plane::runtime::{MachineRuntime, OwnMachines};
use ciproxy_gateway::gateway::MachineGatewayMock;
use ciproxy_gateway::types::Delivery;
use ciproxy_github::assignments::AssignmentService;
use ciproxy_github::persistence::StoreAdapter;
use ciproxy_github::registrations::RunnerRegistrationsMock;
use ciproxy_protocol::agent::ControlMessage;
use ciproxy_protocol::identifiers::JobId;
use ciproxy_protocol::reports::{DrainState, SlotOccupancy, SlotReport};
use ciproxy_protocol::runners;
use ciproxy_store_interface::github::{DemandPhase, GithubJob, RepositoryIdentity, RunFacts};
use ciproxy_store_interface::labels::{Label, WhenNothingFits};
use ciproxy_store_interface::runners::{RunnerPlace, RunnerState};
use ciproxy_store_interface::scale_sets::ScaleSet;
use ciproxy_store_interface::vm_size::VmSize;
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

use super::clock::clock;
use super::support::{Fixture, at};

#[tokio::test]
async fn two_slot_job_reserves_free_indices_zero_and_two_between_busy_jobs() {
    let mut fixture = Fixture::new();
    fixture
        .store
        .update_runner(
            fixture.workspace,
            fixture.organization,
            fixture.intent.runner.id,
            RunnerState::Busy,
            Some(47),
        )
        .unwrap();
    fixture
        .store
        .save_own_slots(fixture.workspace, fixture.intent.runner.id, &[1])
        .unwrap();
    let mut second = fixture.intent.clone();
    second.runner.id = Uuid::new_v4();
    second.runner.github_id = Some(48);
    second.runner.state = RunnerState::Busy;
    second.runner.place = RunnerPlace::Machine {
        machine_id: fixture.machine,
        first_slot: 3,
    };
    second.slot_indices = vec![3];
    second.command_id = Uuid::new_v4();
    fixture
        .store
        .create_own_runner(fixture.workspace, &second)
        .unwrap();
    let mut machine = fixture.store.machines(fixture.workspace).unwrap().remove(0);
    machine.draining = false;
    machine.warm_runners = 0;
    fixture
        .store
        .update_machine(fixture.workspace, &machine)
        .unwrap();
    fixture.contact.report.hardware.threads = 12;
    fixture.contact.report.hardware.memory_bytes = 48_000_000_000;
    fixture.contact.report.capacity = ciproxy_protocol::resources::Slots(4);
    fixture.contact.report.drain = DrainState::Active;
    let mut first = fixture.contact.report.runners[0].clone();
    first.slot_indices = vec![1];
    first.state = runners::RunnerState::Running {
        job_id: JobId("first-job".to_owned()),
    };
    let mut report = first.clone();
    report.runner_id = ciproxy_protocol::identifiers::RunnerId(second.runner.id.to_string());
    report.slot_indices = vec![3];
    report.state = runners::RunnerState::Running {
        job_id: JobId("second-job".to_owned()),
    };
    fixture.contact.report.slots = vec![
        SlotReport {
            index: 0,
            occupancy: SlotOccupancy::Free,
        },
        SlotReport {
            index: 1,
            occupancy: SlotOccupancy::Job {
                runner_id: first.runner_id.clone(),
                job_id: JobId("first-job".to_owned()),
            },
        },
        SlotReport {
            index: 2,
            occupancy: SlotOccupancy::Free,
        },
        SlotReport {
            index: 3,
            occupancy: SlotOccupancy::Job {
                runner_id: report.runner_id.clone(),
                job_id: JobId("second-job".to_owned()),
            },
        },
    ];
    fixture.contact.report.runners = vec![first, report];
    fixture
        .store
        .record_contact(fixture.workspace, fixture.machine, &fixture.contact)
        .unwrap();
    let label = Label {
        id: Uuid::new_v4(),
        name: "large".to_owned(),
        size: VmSize {
            vcpus: 4,
            memory_gb: 16,
            disk_gb: 80,
        },
        allow_machines: true,
        allow_boat: false,
        when_nothing_fits: WhenNothingFits::Wait,
    };
    fixture
        .store
        .create_label(fixture.workspace, &label)
        .unwrap();
    let set = ScaleSet {
        id: Uuid::new_v4(),
        organization_id: fixture.organization,
        label_id: label.id,
        github_id: 56,
        runner_group_id: 45,
    };
    fixture
        .store
        .create_scale_set(fixture.workspace, &set)
        .unwrap();
    fixture
        .store
        .save_github_job(
            fixture.workspace,
            &GithubJob {
                id: Uuid::new_v4(),
                organization_id: fixture.organization,
                scale_set_id: set.id,
                label_id: label.id,
                actions_job_id: None,
                request_id: 0,
                run_id: 7,
                repository: "org/repo".to_owned(),
                name: "wide job".to_owned(),
                workflow_ref: "ci.yml".to_owned(),
                phase: DemandPhase::Assigned,
                result: None,
                assigned_at: Some(at()),
                started_at: None,
                completed_at: None,
                runner_github_id: None,
                facts: RunFacts {
                    id: 7,
                    repository: RepositoryIdentity {
                        id: 8,
                        full_name: "org/repo".to_owned(),
                    },
                    event: "push".to_owned(),
                    head_branch: Some("main".to_owned()),
                    head_sha: "fixture".to_owned(),
                    head_repository: None,
                },
            },
            at(),
        )
        .unwrap();
    let commands = Arc::new(Mutex::new(Vec::new()));
    let observed = commands.clone();
    let gateway = Arc::new(Unimock::new(
        MachineGatewayMock::send
            .each_call(matching!(_, _, _))
            .answers_arc(Arc::new(move |_, _, _, message| {
                observed.lock().unwrap().push(message);
                Ok(Delivery::Connected)
            })),
    ));
    let registrations = Arc::new(Unimock::new(
        RunnerRegistrationsMock::create
            .some_call(matching!(_, _))
            .answers(&|_, _, _| {
                Ok(ciproxy_github::scale_sets::JitRegistration {
                    runner_id: 55,
                    configuration: "jit-fixture".into(),
                })
            }),
    ));
    let clock = clock(true);
    let image = ciproxy_protocol::images::ImageDownload {
        image_id: ciproxy_protocol::identifiers::ImageId("fixture".to_owned()),
        download_url: ciproxy_protocol::secret::Secret::new(
            "https://images.example/fixture.tar.zst",
        ),
        sha256: "a".repeat(64),
        size_bytes: 100,
        root_disk_bytes: 60_263_759_872,
    };
    let runtime = MachineRuntime::new(
        fixture.store.clone(),
        gateway,
        registrations,
        Arc::new(AssignmentService {
            store: Arc::new(StoreAdapter::new(fixture.store.clone(), clock.clone())),
        }),
        clock,
        Some(image),
        Arc::new(Unimock::new(())),
    );
    runtime.tick().await.unwrap();
    let rows = fixture.store.own_runners(fixture.workspace).unwrap();
    assert_eq!(
        rows.iter()
            .filter(|intent| intent.runner.github_id == Some(55))
            .map(|intent| intent.slot_indices.clone())
            .collect::<Vec<_>>(),
        vec![vec![0, 2]]
    );
    assert!(
        matches!(&commands.lock().unwrap()[1],ControlMessage::StartRunner{runner,..} if runner.slot_indices==[0,2])
    );
}
