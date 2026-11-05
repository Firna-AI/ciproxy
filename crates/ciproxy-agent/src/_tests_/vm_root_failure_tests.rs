//! Each root failure propagates its typed cause before later work can run.

use ciproxy_protocol::identifiers::ImageId;

use crate::fixtures::vm;
use crate::root_failure_fixtures::{RootFailure, Stage, error_facts, expected_trace, layout};
use crate::vm_root::RootDisk;

#[tokio::test]
async fn root_preparation_stops_at_each_failed_step_and_returns_its_error() {
    let paths = layout();
    let image = paths
        .image(&ImageId("image-1".to_owned()))
        .join("rootfs.ext4");
    let copy = paths.jail_root(&vm("new", false)).join("rootfs.ext4");
    let failure = RootFailure::new();
    for stage in Stage::ALL {
        failure.reset(stage);
        let result = failure.root.prepare(&image, &copy, 100_000_000_000).await;
        assert_eq!(
            result.map_err(error_facts),
            Err(stage.expected_error()),
            "{stage:?}"
        );
        assert_eq!(
            failure.trace(),
            expected_trace(stage, &image, &copy),
            "{stage:?}"
        );
    }
}
