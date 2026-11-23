//! Linux job VMs and Boat sandboxes size their roots from the Linux release only.

use ciproxy_control_plane::runtime::ImageOffer;
use ciproxy_protocol::identifiers::ImageId;
use ciproxy_protocol::images::ImageDownload;
use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::secret::Secret;

use super::overflow_support::Harness;
use super::settled::Settled;

fn release(platform: Platform, root_disk_bytes: u64) -> ImageOffer {
    ImageOffer::unsigned(ImageDownload {
        image_id: ImageId(format!("{platform:?}-release")),
        platform,
        download_url: Secret::new("https://images.example/release"),
        sha256: "fixture".to_owned(),
        size_bytes: 1,
        root_disk_bytes,
    })
}

/// Start one Boat runner with these releases and return its required root bytes.
async fn boat_root(releases: Vec<ImageOffer>) -> u64 {
    let mut harness = Harness::new(false, false).await;
    harness.runtime.images = releases.into_iter().collect();
    harness.queued();
    harness.runtime.settled_tick().await.unwrap();
    harness
        .fixture
        .store
        .overflow_runtime_snapshot(harness.fixture.workspace)
        .unwrap()
        .starts[0]
        .required_root_disk_bytes
}

#[tokio::test]
async fn a_larger_macos_release_leaves_the_boat_root_of_the_linux_release() {
    let linux_only = boat_root(vec![release(Platform::LinuxX64, 70_000_000_000)]).await;
    let both = boat_root(vec![
        release(Platform::MacosArm64, 200_000_000_000),
        release(Platform::LinuxX64, 70_000_000_000),
    ])
    .await;
    assert_eq!(both, linux_only);
    let macos_only = boat_root(vec![release(Platform::MacosArm64, 200_000_000_000)]).await;
    assert_eq!(macos_only, boat_root(Vec::new()).await);
}
