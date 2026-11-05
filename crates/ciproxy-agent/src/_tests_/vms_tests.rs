//! VM lifetimes belong to separate units. Reconnect never launches a second VM.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::task::Poll;

use ciproxy_protocol::guest::AgentToGuest;
use ciproxy_protocol::secret::Secret;
use ciproxy_protocol::version::ProtocolVersion;
use futures_util::{pin_mut, poll};
use tokio::sync;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::Vms;
use crate::fixtures::vm;
use crate::host_lifecycle_fakes::HeldTools;
use crate::host_tools::ToolsMock;
use crate::layout::Layout;
use crate::network::NetworkMock;
use crate::process::{CommandsMock, UnitState};
use crate::vm_files::VmFilesMock;
use crate::vms::LinuxVms;
use crate::vsock::GuestsMock;

fn runtime(mock: Unimock) -> LinuxVms {
    let mock = Arc::new(mock);
    LinuxVms {
        tools: mock.clone(),
        layout: Layout {
            data: PathBuf::from("/fixture"),
            firecracker: PathBuf::from("/tools/firecracker"),
            jailer: PathBuf::from("/tools/jailer"),
        },
        commands: mock.clone(),
        network: mock.clone(),
        files: mock.clone(),
        guests: mock,
        operation: sync::Mutex::new(()),
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
    let arguments = Arc::new(Mutex::new(Vec::new()));
    let recorded = arguments.clone();
    let mock = Unimock::new((
        ToolsMock::prepare.some_call(matching!()).returns(Ok(())),
        VmFilesMock::prepare.some_call(matching!(_)).returns(Ok(())),
        NetworkMock::create.some_call(matching!(_)).returns(Ok(())),
        CommandsMock::run.some_call(matching!((program, args, input) if *program == "systemd-run" && input.is_empty()
            && args.iter().any(|arg| arg == "/tools/jailer") && args.iter().any(|arg| arg == "ciproxy-vm-local-runner.service")
            && args.iter().any(|arg| arg == "/run/netns/cpx1") && args.iter().any(|arg| arg == "100001")
            && args.iter().all(|arg| !arg.contains("one-use-registration")))).answers_arc(Arc::new(move |_, _, args, _| {
                *recorded.lock().unwrap() = args.to_vec(); Ok(Vec::new())
            })),
        GuestsMock::attach.some_call(matching!((_, config) if matches!(config, Some(AgentToGuest::ConfigureRunner { registration, .. })
            if registration.expose_secret() == "one-use-registration"))).returns(Ok(())),
    ));
    let record = vm("runner", false);
    let runtime = runtime(mock);
    runtime
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
    assert!(
        arguments
            .lock()
            .unwrap()
            .iter()
            .any(|value| value == "--property=RequiresMountsFor=/fixture")
    );
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

#[tokio::test]
async fn a_pending_new_pin_preserves_existing_vm_restore_and_stop() {
    let disks = Arc::new(AtomicUsize::new(0));
    let prepared = disks.clone();
    let mock = Unimock::new((
        ToolsMock::prepare.some_call(matching!()).returns(Ok(())),
        VmFilesMock::prepare
            .some_call(matching!(_))
            .answers_arc(Arc::new(move |_, _| {
                prepared.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })),
        NetworkMock::create.some_call(matching!(_)).returns(Ok(())),
        CommandsMock::run
            .some_call(matching!("systemd-run", _, _))
            .returns(Ok(Vec::new())),
        GuestsMock::attach
            .some_call(matching!(_, None))
            .returns(Ok(())),
        GuestsMock::attach
            .some_call(matching!(_, Some(_)))
            .returns(Ok(())),
        GuestsMock::detach.some_call(matching!(_)).returns(Ok(())),
        CommandsMock::unit
            .some_call(matching!(_))
            .returns(Ok(UnitState::Absent)),
        NetworkMock::remove.some_call(matching!(_)).returns(Ok(())),
        VmFilesMock::remove.some_call(matching!(_)).returns(Ok(())),
    ));
    let mut runtime = runtime(mock);
    let (started, ready) = sync::oneshot::channel();
    let (release, barrier) = sync::oneshot::channel();
    let complete = Arc::new(AtomicBool::new(false));
    runtime.tools = Arc::new(HeldTools {
        delegate: runtime.tools.clone(),
        started: sync::Mutex::new(Some(started)),
        release: sync::Mutex::new(Some(barrier)),
        complete: complete.clone(),
    });
    let existing = vm("existing", true);
    let mut fresh = vm("fresh", false);
    fresh.network_index = 2;
    let start = runtime.start(
        &fresh,
        AgentToGuest::ConfigureRunner {
            protocol_version: ProtocolVersion::CURRENT,
            runner_id: fresh.report.runner_id.clone(),
            organization_id: fresh.report.organization_id.clone(),
            registration: Secret::new("one-use-registration"),
            cache: None,
        },
    );
    pin_mut!(start);
    assert!(matches!(poll!(start.as_mut()), Poll::Pending));
    ready.await.unwrap();
    assert_eq!(disks.load(Ordering::SeqCst), 0);
    runtime.restore(&existing).await.unwrap();
    let stop = runtime.stop(&existing);
    pin_mut!(stop);
    assert!(
        matches!(poll!(stop.as_mut()), Poll::Ready(Ok(()))),
        "an old VM must stop while the new pin download is pending"
    );
    assert!(!complete.load(Ordering::SeqCst));
    release.send(()).unwrap();
    start.await.unwrap();
    assert_eq!(disks.load(Ordering::SeqCst), 1);
    assert!(complete.load(Ordering::SeqCst));
}
