//! Labels and machines keep their platform at the scaler boundary.

use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::statistics::Statistics;

use crate::snapshot::build;

use super::fixtures::{at, input};

#[test]
fn macs_keep_their_platform_and_lease_no_docker_bytes() {
    let mut input = input();
    let runnable = vec![input.organizations[0].id];
    input.labels[0].platform = Platform::MacosArm64;
    let machine = &mut input.machines[0];
    machine.machine.platform = Platform::MacosArm64;
    machine.report.as_mut().unwrap().report.docker_disks = Statistics::Unavailable;
    let snapshot = build(&input, &runnable, at(), RootSizing::default());
    assert_eq!(snapshot.labels[0].platform, Platform::MacosArm64);
    assert_eq!(snapshot.machines[0].platform, Platform::MacosArm64);
    assert_eq!(snapshot.machines[0].docker_leased_bytes, Some(0));

    input.machines[0].machine.platform = Platform::LinuxX64;
    let snapshot = build(&input, &runnable, at(), RootSizing::default());
    assert_eq!(snapshot.machines[0].docker_leased_bytes, None, "Linux");
}
