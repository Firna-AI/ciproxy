//! Image download confirmation precedes runner registration.

use super::clock::clock;
use super::support::{Fixture, at};
use ciproxy_control_plane::runtime::{MachineRuntime, OwnMachines};
use ciproxy_gateway::gateway::MachineGatewayMock;
use ciproxy_gateway::types::Delivery;
use ciproxy_github::assignments::AssignmentService;
use ciproxy_github::persistence::StoreAdapter;
use ciproxy_github::registrations::RunnerRegistrationsMock;
use ciproxy_store_interface::runners::RunnerState;
use std::sync::{Arc, Mutex};
use unimock::{MockFn, Unimock, matching};

#[tokio::test]
async fn bootstrap_fetches_once_per_connection_before_registering_without_a_selected_image() {
    let mut fixture = Fixture::new();
    fixture.contact.report.images.available.clear();
    fixture.contact.report.images.newest = None;
    fixture
        .store
        .record_contact(fixture.workspace, fixture.machine, &fixture.contact)
        .unwrap();
    fixture
        .store
        .update_runner(
            fixture.workspace,
            fixture.organization,
            fixture.intent.runner.id,
            RunnerState::Stopped,
            Some(47),
        )
        .unwrap();
    let mut machine = fixture.store.machines(fixture.workspace).unwrap().remove(0);
    machine.draining = false;
    machine.warm_runners = 1;
    fixture
        .store
        .update_machine(fixture.workspace, &machine)
        .unwrap();
    let messages = Arc::new(Mutex::new(Vec::new()));
    let observed = messages.clone();
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
                    runner_id: 48,
                    configuration: "jit-fixture".into(),
                })
            }),
    ));
    let clock = clock(true);
    let image = ciproxy_protocol::images::ImageDownload {
        image_id: ciproxy_protocol::identifiers::ImageId("selected".to_owned()),
        download_url: ciproxy_protocol::secret::Secret::new(
            "https://images.example/selected.tar.zst",
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
        Some(image.clone()),
    );
    runtime.tick().await.unwrap();
    runtime.tick().await.unwrap();
    assert_eq!(messages.lock().unwrap().len(), 3);
    assert!(messages.lock().unwrap()[1..].iter().all(|message| matches!(
        message,
        ciproxy_protocol::agent::ControlMessage::DockerBudget { budget: None, .. }
    )));
    assert!(
        matches!(&messages.lock().unwrap()[0],ciproxy_protocol::agent::ControlMessage::FetchImage{image:download,..} if download==&image)
    );
    assert_eq!(
        fixture.store.own_runners(fixture.workspace).unwrap().len(),
        1
    );
    fixture.contact.report.runners.clear();
    fixture.contact.report.images.newest = Some(image.image_id.clone());
    fixture.contact.report.images.available = vec![ciproxy_protocol::images::ImageMetadata {
        image_id: image.image_id,
        root_disk_bytes: image.root_disk_bytes,
    }];
    fixture.contact.at = at() + chrono::Duration::seconds(1);
    fixture
        .store
        .record_contact(fixture.workspace, fixture.machine, &fixture.contact)
        .unwrap();
    runtime.tick().await.unwrap();
    assert_eq!(messages.lock().unwrap().len(), 5);
    assert!(
        matches!(&messages.lock().unwrap()[4],ciproxy_protocol::agent::ControlMessage::StartRunner{runner,..} if runner.organization_id.0=="42")
    );
    assert_eq!(
        fixture
            .store
            .own_runners(fixture.workspace)
            .unwrap()
            .iter()
            .filter(|runner| runner.runner.state == RunnerState::Starting)
            .count(),
        1
    );
}
