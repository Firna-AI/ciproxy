//! SSH, clones, and the guest start for the macOS VM tests.

use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, Instant};

use ciproxy_vz::lease::{LEASE_FILE, lookup};
use ciproxy_vz::{Bundle, Identity, Vm, VmSettings, clone_bundle};

use super::{PORT, TIMEOUT};

pub fn variable(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("set {name}"))
}

pub fn ssh(address: Ipv4Addr, command: &str) -> Output {
    Command::new("ssh")
        .args([
            "-o",
            "StrictHostKeyChecking=no",
            "-o",
            "UserKnownHostsFile=/dev/null",
        ])
        .args(["-o", "BatchMode=yes", "-o", "ConnectTimeout=5"])
        .arg(format!("{}@{address}", variable("CIPROXY_VZ_GUEST_USER")))
        .arg(command)
        .output()
        .unwrap()
}

fn scp(address: Ipv4Addr, file: &Path, name: &str) {
    let status = Command::new("scp")
        .args([
            "-o",
            "StrictHostKeyChecking=no",
            "-o",
            "UserKnownHostsFile=/dev/null",
        ])
        .args(["-o", "BatchMode=yes"])
        .arg(file)
        .arg(format!(
            "{}@{address}:{name}",
            variable("CIPROXY_VZ_GUEST_USER")
        ))
        .status()
        .unwrap();
    assert!(status.success(), "scp {name} failed");
}

/// A clone that is removed on drop.
pub struct Clone(pub Bundle);

impl Clone {
    pub fn new(source: &Bundle, name: &str, identity: &Identity) -> Self {
        let target = PathBuf::from(variable("CIPROXY_VZ_WORK"))
            .join(format!("guest-{name}-{}", std::process::id()));
        Self(clone_bundle(source, &target, identity).unwrap())
    }
}

impl Drop for Clone {
    fn drop(&mut self) {
        let _ignored = std::fs::remove_dir_all(self.0.root());
    }
}

pub fn settings() -> VmSettings {
    VmSettings {
        cpus: 4,
        memory_bytes: 4 << 30,
        display: None,
    }
}

pub fn guest_address(vm: &Vm, since: Instant) -> Ipv4Addr {
    loop {
        assert!(since.elapsed() < TIMEOUT, "no SSH");
        if let Some(address) = lookup(LEASE_FILE.as_ref(), vm.mac_address()).unwrap()
            && ssh(address, "true").status.success()
        {
            return address;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}

/// Copy the guest and the fake entry over SSH, and start the guest as root.
pub fn start_guest(address: Ipv4Addr) {
    let user = variable("CIPROXY_VZ_GUEST_USER");
    let entry = std::env::temp_dir().join(format!("ciproxy-fake-entry-{}", std::process::id()));
    let script = include_str!("../fixtures/macos-fake-entry.sh").replace("EXPECTED_USER", &user);
    std::fs::write(&entry, script).unwrap();
    scp(address, &entry, "fake-entry");
    let _ignored = std::fs::remove_file(entry);
    scp(
        address,
        Path::new(&variable("CIPROXY_GUEST_BINARY")),
        "ciproxy-guest",
    );
    let start = format!(
        "chmod 0755 fake-entry ciproxy-guest && sudo -n env LEAK=x \
         nohup \"$HOME/ciproxy-guest\" --image-id e2e --runner-account {user} \
         --runner-entry \"$HOME/fake-entry\" vsock --port {PORT} \
         </dev/null >guest.log 2>&1 &"
    );
    assert!(ssh(address, &start).status.success());
}
