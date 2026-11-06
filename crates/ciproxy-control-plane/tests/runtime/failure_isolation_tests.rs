//! One suspended organization cannot block another organization's runner starts.

use std::sync::Arc;

use ciproxy_control_plane::runtime::{MachineRuntime, OwnMachines};
use ciproxy_gateway::gateway::MachineGatewayMock;
use ciproxy_gateway::types::Delivery;
use ciproxy_github::assignments::AssignmentService;
use ciproxy_github::persistence::StoreAdapter;
use ciproxy_github::registrations::RunnerRegistrationsMock;
use ciproxy_store_interface::organizations::AppState;
use ciproxy_store_interface::runners::RunnerState;
use ciproxy_store_interface::scale_sets::ScaleSet;
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

use super::clock::clock;
use super::support::Fixture;

#[tokio::test]
async fn failed_retirement_does_not_block_an_active_organizations_start() {
    let mut fixture = Fixture::new();
    let mut other = fixture
        .store
        .organization(fixture.workspace, fixture.organization)
        .unwrap()
        .unwrap();
    other.id = Uuid::new_v4();
    other.github_id = 90;
    other.installation_id = 91;
    other.login = "other".to_owned();
    other.app_state = AppState::Active;
    fixture.store.install_organization(&other).unwrap();
    fixture
        .store
        .create_scale_set(
            fixture.workspace,
            &ScaleSet {
                id: Uuid::new_v4(),
                organization_id: other.id,
                label_id: fixture.intent.runner.label_id,
                github_id: 92,
                runner_group_id: 93,
            },
        )
        .unwrap();
    fixture
        .store
        .set_app_suspended(fixture.workspace, fixture.organization, true)
        .unwrap();
    let new_machine = fixture.add_machine();
    let registrations = Arc::new(Unimock::new((
        RunnerRegistrationsMock::remove
            .some_call(matching!(_, 47))
            .returns(Err(ciproxy_github::error::Error::AccessUnavailable(403))),
        RunnerRegistrationsMock::create
            .some_call(matching!(_, _))
            .answers(&|_, _, _| {
                Ok(ciproxy_github::scale_sets::JitRegistration {
                    runner_id: 94,
                    configuration: "jit-fixture".into(),
                })
            }),
    )));
    let gateway = Arc::new(Unimock::new(
        MachineGatewayMock::send
            .each_call(matching!(_, _, _))
            .answers_arc(Arc::new(move |_, _, machine, message| {
                if matches!(
                    message,
                    ciproxy_protocol::agent::ControlMessage::StartRunner { .. }
                ) {
                    assert_eq!(machine, new_machine);
                }
                Ok(Delivery::Connected)
            })),
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
    );
    runtime.tick().await.unwrap();
    let rows = fixture.store.own_runners(fixture.workspace).unwrap();
    assert_eq!(
        rows.iter()
            .filter(|intent| intent.runner.organization_id == other.id
                && intent.runner.github_id == Some(94)
                && intent.runner.state == RunnerState::Starting)
            .count(),
        1
    );
    assert_eq!(
        rows.iter()
            .filter(
                |intent| intent.runner.organization_id == fixture.organization
                    && intent.runner.state == RunnerState::Ready
            )
            .count(),
        1
    );
}
