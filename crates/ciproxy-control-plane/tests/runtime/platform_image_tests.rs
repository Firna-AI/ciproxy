//! Each machine gets the image release of its own platform, and only that one.

use super::clock::clock;
use super::settled::Settled;
use super::support::Fixture;
use ciproxy_control_plane::runtime::MachineRuntime;
use ciproxy_gateway::gateway::MachineGatewayMock;
use ciproxy_gateway::types::Delivery;
use ciproxy_github::assignments::AssignmentService;
use ciproxy_github::persistence::StoreAdapter;
use ciproxy_protocol::agent::ControlMessage;
use ciproxy_protocol::identifiers::ImageId;
use ciproxy_protocol::images::ImageDownload;
use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::runners::RunnerState;
use std::sync::{Arc, Mutex};
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

fn release(platform: Platform, name: &str) -> ImageDownload {
    ImageDownload {
        image_id: ImageId(name.to_owned()),
        platform,
        download_url: Secret::new(format!("https://images.example/{name}.tar")),
        sha256: "a".repeat(64),
        size_bytes: 100,
        root_disk_bytes: 60_263_759_872,
    }
}

/// Run one tick with a Linux machine and a Mac, and return each image fetch.
async fn fetches(images: Vec<ImageDownload>) -> (Fixture, Uuid, Vec<(Uuid, ImageId)>) {
    let mut fixture = Fixture::new();
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
    let mac = fixture.add_machine_on(Platform::MacosArm64);
    let mut machine = fixture
        .store
        .machines(fixture.workspace)
        .unwrap()
        .into_iter()
        .find(|machine| machine.id == mac)
        .unwrap();
    machine.draining = true;
    machine.warm_runners = 0;
    fixture
        .store
        .update_machine(fixture.workspace, &machine)
        .unwrap();
    let sent = Arc::new(Mutex::new(Vec::new()));
    let observed = sent.clone();
    let gateway = Arc::new(Unimock::new(
        MachineGatewayMock::send
            .each_call(matching!(_, _, _))
            .answers_arc(Arc::new(move |_, _, machine, message| {
                if let ControlMessage::FetchImage { image, .. } = message {
                    observed.lock().unwrap().push((machine, image.image_id));
                }
                Ok(Delivery::Connected)
            })),
    ));
    let clock = clock(true);
    let runtime = MachineRuntime::new(
        fixture.store.clone(),
        gateway,
        Arc::new(Unimock::new(())),
        Arc::new(AssignmentService {
            store: Arc::new(StoreAdapter::new(fixture.store.clone(), clock.clone())),
        }),
        clock,
        images,
        Arc::new(Unimock::new(())),
    );
    runtime.settled_tick().await.unwrap();
    let mut fetched = sent.lock().unwrap().clone();
    fetched.sort_by_key(|(machine, _)| *machine != fixture.machine);
    (fixture, mac, fetched)
}

#[tokio::test]
async fn a_mac_gets_the_macos_release_and_a_linux_machine_the_linux_release() {
    let (fixture, mac, fetched) = fetches(vec![
        release(Platform::LinuxX64, "linux-release"),
        release(Platform::MacosArm64, "macos-release"),
    ])
    .await;
    assert_eq!(
        fetched,
        vec![
            (fixture.machine, ImageId("linux-release".to_owned())),
            (mac, ImageId("macos-release".to_owned())),
        ]
    );
}

#[tokio::test]
async fn a_mac_gets_no_image_while_only_a_linux_release_exists() {
    let (fixture, _, fetched) = fetches(vec![release(Platform::LinuxX64, "linux-release")]).await;
    assert_eq!(
        fetched,
        vec![(fixture.machine, ImageId("linux-release".to_owned()))]
    );
}
