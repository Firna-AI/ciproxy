//! VM lifetimes belong to separate units. Reconnect never launches a second VM.

use std::path::PathBuf;
use std::sync::Arc;

use ciproxy_protocol::guest::AgentToGuest;
use ciproxy_protocol::secret::Secret;
use ciproxy_protocol::version::ProtocolVersion;
use tokio::sync::Mutex;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::Vms;
use crate::fixtures::vm;
use crate::layout::Layout;
use crate::network::NetworkMock;
use crate::process::{CommandsMock, UnitState};
use crate::vm_files::VmFilesMock;
use crate::vms::LinuxVms;
use crate::vsock::GuestsMock;

fn runtime(mock: Unimock) -> LinuxVms {
    let mock = Arc::new(mock);
    LinuxVms {
        layout: Layout {
            data: PathBuf::from("/fixture"),
            firecracker: PathBuf::from("/tools/firecracker"),
            jailer: PathBuf::from("/tools/jailer"),
        },
        commands: mock.clone(),
        network: mock.clone(),
        files: mock.clone(),
        guests: mock,
        operation: Mutex::new(()),
    }
}

#[tokio::test]
async fn request_stop_sends_shutdown_without_killing_the_unit_or_disks() {
    let mock = Unimock::new(
        GuestsMock::send
            .some_call(matching!((_, message) if **message == AgentToGuest::Shutdown))
            .returns(Ok(())),
    );
    runtime(mock)
        .request_stop(&vm("raced", false))
        .await
        .unwrap();
}

#[tokio::test]
async fn starts_under_jailer_in_an_independent_unit_with_registration_only_on_vsock() {
    let mock = Unimock::new((
        VmFilesMock::prepare.some_call(matching!(_)).returns(Ok(())),
        NetworkMock::create.some_call(matching!(_)).returns(Ok(())),
        CommandsMock::run.some_call(matching!((program, args, input) if *program == "systemd-run" && input.is_empty()
            && args.iter().any(|arg| arg == "/tools/jailer") && args.iter().any(|arg| arg == "ciproxy-vm-local-runner.service")
            && args.iter().any(|arg| arg == "/run/netns/cpx1") && args.iter().any(|arg| arg == "100001")
            && args.iter().all(|arg| !arg.contains("one-use-registration")))).returns(Ok(Vec::new())),
        GuestsMock::attach.some_call(matching!((_, config) if matches!(config, Some(AgentToGuest::ConfigureRunner { registration, .. })
            if registration.expose_secret() == "one-use-registration"))).returns(Ok(())),
    ));
    let record = vm("runner", false);
    runtime(mock)
        .start(
            &record,
            AgentToGuest::ConfigureRunner {
                protocol_version: ProtocolVersion::CURRENT,
                runner_id: record.report.runner_id.clone(),
                organization_id: record.report.organization_id.clone(),
                registration: Secret::new("one-use-registration"),
                cache: None,
            },
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn stop_discards_disks_even_when_the_owned_unit_already_exited() {
    let mock = Unimock::new((
        GuestsMock::detach.some_call(matching!(_)).returns(Ok(())),
        CommandsMock::unit
            .some_call(matching!(_))
            .returns(Ok(UnitState::Absent)),
        NetworkMock::remove.some_call(matching!(_)).returns(Ok(())),
        VmFilesMock::remove.some_call(matching!(_)).returns(Ok(())),
    ));
    runtime(mock).stop(&vm("finished", true)).await.unwrap();
}
