//! Each machine gets the image release of its own platform, and only that one.

use super::clock::clock;
use super::settled::Settled;
use super::support::Fixture;
use ciproxy_control_plane::image_signing::{CdnSigning, PlatformSigning};
use ciproxy_control_plane::macos_release::MacosImageSettings;
use ciproxy_control_plane::runtime::{ImageOffer, MachineRuntime};
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
async fn fetches(images: Vec<ImageOffer>) -> (Fixture, Uuid, Vec<(Uuid, ImageId, String)>) {
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
                    let url = image.download_url.expose_secret().to_owned();
                    observed
                        .lock()
                        .unwrap()
                        .push((machine, image.image_id, url));
                }
                Ok(Delivery::Connected)
            })),
    ));
    let clock = clock(true);
    let mut runtime = MachineRuntime::new(
        fixture.store.clone(),
        gateway,
        Arc::new(Unimock::new(())),
        Arc::new(AssignmentService {
            store: Arc::new(StoreAdapter::new(fixture.store.clone(), clock.clone())),
        }),
        clock,
        Vec::new(),
        Arc::new(Unimock::new(())),
    );
    runtime.images = images;
    runtime.settled_tick().await.unwrap();
    let mut fetched = sent.lock().unwrap().clone();
    fetched.sort_by_key(|(machine, _, _)| *machine != fixture.machine);
    (fixture, mac, fetched)
}

/// The released image of each machine, with no signature.
fn unsigned(images: Vec<ImageDownload>) -> Vec<ImageOffer> {
    images.into_iter().map(ImageOffer::unsigned).collect()
}

fn ids(fetched: Vec<(Uuid, ImageId, String)>) -> Vec<(Uuid, ImageId)> {
    fetched
        .into_iter()
        .map(|(machine, id, _)| (machine, id))
        .collect()
}

#[tokio::test]
async fn a_mac_gets_the_macos_release_and_a_linux_machine_the_linux_release() {
    let (fixture, mac, fetched) = fetches(unsigned(vec![
        release(Platform::LinuxX64, "linux-release"),
        release(Platform::MacosArm64, "macos-release"),
    ]))
    .await;
    assert_eq!(
        ids(fetched),
        vec![
            (fixture.machine, ImageId("linux-release".to_owned())),
            (mac, ImageId("macos-release".to_owned())),
        ]
    );
}

#[tokio::test]
async fn a_mac_gets_no_image_while_only_a_linux_release_exists() {
    let (fixture, _, fetched) =
        fetches(unsigned(vec![release(Platform::LinuxX64, "linux-release")])).await;
    assert_eq!(
        ids(fetched),
        vec![(fixture.machine, ImageId("linux-release".to_owned()))]
    );
}

#[tokio::test]
async fn a_mac_gets_a_two_hour_presigned_r2_url_and_a_linux_machine_a_cdn_url() {
    let macos = MacosImageSettings {
        url_prefix: Some("https://account.r2.example/ciproxy-macos-images".to_owned()),
        manifest_path: Some("releases/macos-release/manifest.json".to_owned()),
        key: Some(Secret::new(
            r#"{"access_key_id": "fixture-key-id", "secret_access_key": "fixture-secret"}"#,
        )),
    };
    let signing = PlatformSigning {
        linux: Arc::new(
            CdnSigning::new("test-key".to_owned(), "AAECAwQFBgcICQoLDA0ODw==").unwrap(),
        ),
        macos: macos.download_signing(),
    };
    let (_, _, fetched) = fetches(signing.offers(&[
        release(Platform::LinuxX64, "linux-release"),
        release(Platform::MacosArm64, "macos-release"),
    ]))
    .await;
    let urls: Vec<&str> = fetched.iter().map(|(_, _, url)| url.as_str()).collect();
    assert_eq!(urls.len(), 2);
    assert!(
        urls[0].starts_with("https://images.example/linux-release.tar?Expires=")
            && urls[0].contains("&KeyName=test-key&Signature="),
        "{}",
        urls[0]
    );
    let (unsigned, signature) = urls[1].split_once("&X-Amz-Signature=").unwrap();
    assert_eq!(
        unsigned,
        "https://images.example/macos-release.tar?X-Amz-Algorithm=AWS4-HMAC-SHA256\
         &X-Amz-Credential=fixture-key-id%2F20261105%2Fauto%2Fs3%2Faws4_request\
         &X-Amz-Date=20261105T120000Z&X-Amz-Expires=7200&X-Amz-SignedHeaders=host"
    );
    assert_eq!(signature.len(), 64);
}
