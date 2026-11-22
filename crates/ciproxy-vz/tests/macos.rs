//! Tests on a real Apple Silicon Mac. They start real macOS VMs, so they
//! are ignored by default. See "Tests on a Mac" in the crate README.
#![cfg(target_os = "macos")]

mod support;

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

use ciproxy_vz::{Error, StopReason, Vm, VmState};
use futures_executor::block_on;
use support::{Clone, env_path, scp, settings, ssh, wait_for_ssh};

const BOOT_TIMEOUT: Duration = Duration::from_secs(120);
const UNUSED_PORT: u32 = 40_404;
const ECHO_PORT: u32 = 5252;

/// A headless VM, with no AppKit and no `NSApplication`, runs a full cycle.
#[test]
#[ignore = "starts macOS VMs on a Mac"]
fn headless_vm_starts_saves_restores_and_stops() {
    let clone = Clone::fresh("cycle");
    let state = clone.bundle.root().join("state.vzvmsave");
    let vm = Vm::new(&clone.bundle, &settings()).unwrap();
    assert_eq!(vm.state(), VmState::Stopped);

    let started = Instant::now();
    block_on(vm.start()).unwrap();
    assert_eq!(vm.state(), VmState::Running);
    let address = wait_for_ssh(vm.mac_address(), started, BOOT_TIMEOUT);
    eprintln!("cold start to SSH at {address}: {:?}", started.elapsed());

    let refused = block_on(vm.connect(UNUSED_PORT));
    assert!(
        matches!(refused, Err(Error::GuestNotListening { port: UNUSED_PORT })),
        "{refused:?}"
    );

    block_on(vm.pause()).unwrap();
    let saving = Instant::now();
    block_on(vm.save(&state)).unwrap();
    eprintln!(
        "save: {:?}, {} bytes",
        saving.elapsed(),
        state.metadata().unwrap().len()
    );
    block_on(vm.stop()).unwrap();
    assert_eq!(block_on(vm.stopped()), StopReason::Host);
    drop(vm);

    let vm = Vm::new(&clone.bundle, &settings()).unwrap();
    let restoring = Instant::now();
    block_on(vm.restore(&state)).unwrap();
    assert_eq!(vm.state(), VmState::Paused);
    block_on(vm.resume()).unwrap();
    wait_for_ssh(vm.mac_address(), restoring, BOOT_TIMEOUT);
    eprintln!("restore to SSH: {:?}", restoring.elapsed());

    block_on(vm.stop()).unwrap();
    assert_eq!(vm.state(), VmState::Stopped);
    assert_eq!(block_on(vm.stopped()), StopReason::Host);
}

/// macOS runs at most 2 macOS VMs, and the third start has its own error.
#[test]
#[ignore = "starts macOS VMs on a Mac"]
fn third_vm_fails_with_the_limit_error() {
    let clones = [
        Clone::fresh("limit-a"),
        Clone::fresh("limit-b"),
        Clone::fresh("limit-c"),
    ];
    let vms: Vec<Vm> = clones
        .iter()
        .map(|clone| Vm::new(&clone.bundle, &settings()).unwrap())
        .collect();
    block_on(vms[0].start()).unwrap();
    block_on(vms[1].start()).unwrap();
    let third = block_on(vms[2].start());
    assert!(
        matches!(third, Err(Error::VirtualMachineLimit { .. })),
        "{third:?}"
    );
    eprintln!("third start: {}", third.unwrap_err());

    block_on(vms[1].pause()).unwrap();
    let while_paused = block_on(vms[2].start());
    eprintln!("third start while one VM is paused: {while_paused:?}");
    for vm in &vms {
        if vm.state() != VmState::Stopped {
            block_on(vm.stop()).unwrap();
        }
    }
}

/// The vsock stream stays open after the framework drops its connection,
/// and a refused connect turns into a working one once the guest listens.
#[test]
#[ignore = "starts a macOS VM on a Mac and needs SSH into the guest"]
fn vsock_stream_outlives_the_connection_object() {
    let clone = Clone::fresh("vsock");
    let vm = Vm::new(&clone.bundle, &settings()).unwrap();
    block_on(vm.start()).unwrap();
    let address = wait_for_ssh(vm.mac_address(), Instant::now(), BOOT_TIMEOUT);

    scp(address, &env_path("CIPROXY_VZ_GUEST_ECHO"));
    let output = ssh(
        address,
        &format!("nohup ./guest_vsock_echo {ECHO_PORT} >/dev/null 2>&1 &"),
    );
    assert!(output.status.success(), "{output:?}");

    let since = Instant::now();
    let fd = loop {
        match block_on(vm.connect(ECHO_PORT)) {
            Ok(fd) => break fd,
            Err(Error::GuestNotListening { .. }) if since.elapsed() < BOOT_TIMEOUT => {
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(error) => panic!("{error}"),
        }
    };
    // Give the framework time to release its connection object.
    std::thread::sleep(Duration::from_secs(2));
    let mut stream = UnixStream::from(fd);
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    stream.write_all(b"ping\n").unwrap();
    let mut reply = [0_u8; 5];
    stream.read_exact(&mut reply).unwrap();
    assert_eq!(&reply, b"ping\n");

    block_on(vm.stop()).unwrap();
}

/// `shutdown -h now` in the guest reports a guest stop.
#[test]
#[ignore = "starts a macOS VM on a Mac and needs SSH into the guest"]
fn guest_shutdown_reports_a_guest_stop() {
    let clone = Clone::fresh("shutdown");
    let vm = Vm::new(&clone.bundle, &settings()).unwrap();
    block_on(vm.start()).unwrap();
    let address = wait_for_ssh(vm.mac_address(), Instant::now(), BOOT_TIMEOUT);
    let stopping = Instant::now();
    let _connection_may_drop = ssh(address, "sudo -n shutdown -h now");
    assert_eq!(block_on(vm.stopped()), StopReason::Guest);
    eprintln!("guest shutdown to stop: {:?}", stopping.elapsed());
}
