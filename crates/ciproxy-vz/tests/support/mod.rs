//! Shared parts of the macOS tests. See the crate README for the setup.

use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use ciproxy_vz::lease::{LEASE_FILE, lookup};
use ciproxy_vz::{Bundle, Identity, MacAddress, VmSettings, clone_bundle};

const GIB: u64 = 1024 * 1024 * 1024;

/// A clone of the base bundle that is removed on drop.
pub struct Clone {
    pub bundle: Bundle,
}

impl Clone {
    /// Clone `CIPROXY_VZ_BASE` into `CIPROXY_VZ_WORK` with a fresh identity.
    pub fn fresh(name: &str) -> Self {
        Self::with_identity(name, &Identity::fresh().unwrap())
    }

    pub fn with_identity(name: &str, identity: &Identity) -> Self {
        let base = Bundle::new(env_path("CIPROXY_VZ_BASE"));
        let target = env_path("CIPROXY_VZ_WORK").join(format!("{name}-{}", std::process::id()));
        let started = Instant::now();
        let bundle = clone_bundle(&base, &target, identity).unwrap();
        eprintln!("clone {name}: {:?}", started.elapsed());
        Self { bundle }
    }
}

impl Drop for Clone {
    fn drop(&mut self) {
        let _ignored = std::fs::remove_dir_all(self.bundle.root());
    }
}

pub fn env_path(name: &str) -> PathBuf {
    std::env::var_os(name)
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("set {name}; see the crate README"))
}

pub fn settings() -> VmSettings {
    VmSettings {
        cpus: 4,
        memory_bytes: 4 * GIB,
        display: None,
    }
}

/// Wait for a DHCP lease of `mac`, then for SSH on it.
pub fn wait_for_ssh(mac: MacAddress, since: Instant, timeout: Duration) -> Ipv4Addr {
    loop {
        assert!(since.elapsed() < timeout, "no SSH for {mac} in {timeout:?}");
        if let Some(address) = lookup(LEASE_FILE.as_ref(), mac).unwrap()
            && std::net::TcpStream::connect_timeout(&(address, 22).into(), Duration::from_secs(1))
                .is_ok()
        {
            return address;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// Run a command in the guest over SSH as `CIPROXY_VZ_GUEST_USER`.
pub fn ssh(address: Ipv4Addr, command: &str) -> std::process::Output {
    let user = std::env::var("CIPROXY_VZ_GUEST_USER").expect("set CIPROXY_VZ_GUEST_USER");
    Command::new("ssh")
        .args([
            "-o",
            "StrictHostKeyChecking=no",
            "-o",
            "UserKnownHostsFile=/dev/null",
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=5",
        ])
        .arg(format!("{user}@{address}"))
        .arg(command)
        .output()
        .unwrap()
}

/// Copy a file into the guest's home directory over SCP.
pub fn scp(address: Ipv4Addr, file: &std::path::Path) {
    let user = std::env::var("CIPROXY_VZ_GUEST_USER").expect("set CIPROXY_VZ_GUEST_USER");
    let status = Command::new("scp")
        .args([
            "-o",
            "StrictHostKeyChecking=no",
            "-o",
            "UserKnownHostsFile=/dev/null",
            "-o",
            "BatchMode=yes",
        ])
        .arg(file)
        .arg(format!("{user}@{address}:"))
        .status()
        .unwrap();
    assert!(status.success(), "scp failed");
}
