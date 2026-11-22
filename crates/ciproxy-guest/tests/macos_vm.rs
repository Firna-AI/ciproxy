//! The guest in a real macOS VM, over the virtio socket of
//! Virtualization.framework. Ignored by default, because it starts a VM.
//!
//! It needs an Apple Silicon Mac and these variables:
//! `CIPROXY_VZ_BASE` (an installed bundle with SSH by key and `sudo -n`),
//! `CIPROXY_VZ_WORK` (a directory on the same APFS volume),
//! `CIPROXY_VZ_GUEST_USER` (the SSH user), and `CIPROXY_GUEST_BINARY` (the
//! `ciproxy-guest` built for `aarch64-apple-darwin`). Run it with
//! `CARGO_TARGET_AARCH64_APPLE_DARWIN_RUNNER=crates/ciproxy-vz/sign-and-run.sh`
//! `cargo test --package ciproxy-guest --test macos_vm -- --ignored --nocapture`.
#![cfg(target_os = "macos")]

use std::io::{BufRead, BufReader, Write};
use std::net::Ipv4Addr;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{Duration, Instant};

use ciproxy_protocol::guest::{AgentToGuest, GuestToAgent, JobDecision, JobPreparation};
use ciproxy_protocol::identifiers::{JobId, OrganizationId, RunnerId};
use ciproxy_protocol::secret::Secret;
use ciproxy_protocol::version::ProtocolVersion;
use ciproxy_vz::lease::{LEASE_FILE, lookup};
use ciproxy_vz::{Bundle, Error, Identity, StopReason, Vm, VmSettings, clone_bundle};
use futures_executor::block_on;

const TIMEOUT: Duration = Duration::from_secs(120);
const PORT: u32 = 5252;
const JOB: &str = "00000000-0000-0000-0000-000000000123";

fn variable(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("set {name}"))
}

fn ssh(address: Ipv4Addr, command: &str) -> Output {
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

fn scp(address: Ipv4Addr, file: &std::path::Path, name: &str) {
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

struct Clone(Bundle);

impl Drop for Clone {
    fn drop(&mut self) {
        let _ignored = std::fs::remove_dir_all(self.0.root());
    }
}

struct Agent {
    reader: BufReader<UnixStream>,
    writer: UnixStream,
}

impl Agent {
    fn receive(&mut self) -> GuestToAgent {
        let mut line = String::new();
        self.reader.read_line(&mut line).unwrap();
        eprintln!("guest: {}", line.trim_end());
        serde_json::from_str(&line).unwrap()
    }

    fn send(&mut self, message: &AgentToGuest) {
        let mut bytes = serde_json::to_vec(message).unwrap();
        bytes.push(b'\n');
        self.writer.write_all(&bytes).unwrap();
    }
}

/// Configure a runner over vsock, hold its job for a decision, and see the
/// guest report the exit and power the VM off.
#[test]
#[ignore = "starts a macOS VM on a Mac and needs SSH into the guest"]
fn guest_runs_one_runner_over_vsock_and_powers_off() {
    let user = variable("CIPROXY_VZ_GUEST_USER");
    let target = PathBuf::from(variable("CIPROXY_VZ_WORK"))
        .join(format!("guest-e2e-{}", std::process::id()));
    let base = Bundle::new(variable("CIPROXY_VZ_BASE"));
    let clone = Clone(clone_bundle(&base, &target, &Identity::fresh().unwrap()).unwrap());
    let settings = VmSettings {
        cpus: 4,
        memory_bytes: 4 << 30,
        display: None,
    };
    let vm = Vm::new(&clone.0, &settings).unwrap();
    let started = Instant::now();
    block_on(vm.start()).unwrap();
    let address = loop {
        assert!(started.elapsed() < TIMEOUT, "no SSH");
        if let Some(address) = lookup(LEASE_FILE.as_ref(), vm.mac_address()).unwrap()
            && ssh(address, "true").status.success()
        {
            break address;
        }
        std::thread::sleep(Duration::from_millis(500));
    };

    let entry = std::env::temp_dir().join(format!("ciproxy-fake-entry-{}", std::process::id()));
    std::fs::write(
        &entry,
        include_str!("fixtures/macos-fake-entry.sh").replace("EXPECTED_USER", &user),
    )
    .unwrap();
    scp(address, &entry, "fake-entry");
    scp(
        address,
        &PathBuf::from(variable("CIPROXY_GUEST_BINARY")),
        "ciproxy-guest",
    );
    let start = format!(
        "chmod 0755 fake-entry ciproxy-guest && sudo -n env LEAK=x \
         nohup \"$HOME/ciproxy-guest\" --image-id e2e --runner-account {user} \
         --runner-entry \"$HOME/fake-entry\" vsock --port {PORT} \
         </dev/null >guest.log 2>&1 &"
    );
    assert!(ssh(address, &start).status.success());

    let fd = loop {
        match block_on(vm.connect(PORT)) {
            Ok(fd) => break fd,
            Err(Error::GuestNotListening { .. }) if started.elapsed() < TIMEOUT => {
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(error) => panic!("{error}"),
        }
    };
    let stream = UnixStream::from(fd);
    stream
        .set_read_timeout(Some(Duration::from_secs(60)))
        .unwrap();
    let mut agent = Agent {
        reader: BufReader::new(stream.try_clone().unwrap()),
        writer: stream,
    };
    let runner = RunnerId("runner-e2e".into());
    assert!(matches!(
        agent.receive(),
        GuestToAgent::Hello {
            handoff_protocol_version: 2,
            configuration_received: false,
            ..
        }
    ));
    agent.send(&AgentToGuest::ConfigureRunner {
        protocol_version: ProtocolVersion::CURRENT,
        runner_id: runner.clone(),
        organization_id: OrganizationId("org-e2e".into()),
        registration: Secret::new("fake-jit-for-the-vm-test"),
    });
    let exit = loop {
        match agent.receive() {
            GuestToAgent::JobStarting {
                job_id, runtime, ..
            } => {
                assert_eq!(job_id, JobId(JOB.into()));
                assert_eq!(runtime.proxy_url, "http://127.0.0.1:8230");
                assert_eq!(runtime.docker_data_bytes, 0);
                assert_eq!(runtime.github_token.expose_secret(), "original-token");
                agent.send(&AgentToGuest::PrepareJob {
                    job_id,
                    decision: JobDecision::GitHubOnly,
                });
            }
            GuestToAgent::JobPrepared { outcome, .. } => {
                assert_eq!(outcome, JobPreparation::default());
            }
            GuestToAgent::RunnerExited { exit, .. } => break exit,
            _ => {}
        }
    };
    if exit.code != Some(3) {
        let log = ssh(address, "cat guest.log");
        panic!(
            "fake entry failed with {exit:?}; see its exit codes. Guest log:\n{}",
            String::from_utf8_lossy(&log.stdout)
        );
    }
    let stopping = Instant::now();
    assert_eq!(block_on(vm.stopped()), StopReason::Guest);
    eprintln!("runner exit to VM stop: {:?}", stopping.elapsed());
    let _ignored = std::fs::remove_file(entry);
}
