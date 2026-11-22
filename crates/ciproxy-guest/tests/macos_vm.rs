//! The guest in a real macOS VM, over the virtio socket of
//! Virtualization.framework. Ignored by default, because it starts VMs.
//!
//! It needs an Apple Silicon Mac and these variables:
//! `CIPROXY_VZ_BASE` (an installed bundle with SSH by key and `sudo -n`),
//! `CIPROXY_VZ_WORK` (a directory on the same APFS volume),
//! `CIPROXY_VZ_GUEST_USER` (the SSH user), and `CIPROXY_GUEST_BINARY` (the
//! `ciproxy-guest` built for `aarch64-apple-darwin`). Run it with
//! `CARGO_TARGET_AARCH64_APPLE_DARWIN_RUNNER=crates/ciproxy-vz/sign-and-run.sh`
//! `cargo test --package ciproxy-guest --test macos_vm -- --ignored --nocapture --test-threads=1`.
#![cfg(target_os = "macos")]

mod macos_vm_support;

use std::io::{BufRead, BufReader, Write};
use std::net::Ipv4Addr;
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

use ciproxy_protocol::guest::{AgentToGuest, GuestToAgent, JobDecision, JobPreparation};
use ciproxy_protocol::identifiers::{JobId, OrganizationId, RunnerId};
use ciproxy_protocol::runners::RunnerExit;
use ciproxy_protocol::secret::Secret;
use ciproxy_protocol::version::ProtocolVersion;
use ciproxy_vz::{Bundle, Error, Identity, StopReason, Vm};
use futures_executor::block_on;
use macos_vm_support::{Clone, guest_address, settings, ssh, start_guest, variable};

const TIMEOUT: Duration = Duration::from_secs(120);
const PORT: u32 = 5252;
const JOB: &str = "00000000-0000-0000-0000-000000000123";

struct Agent {
    reader: BufReader<UnixStream>,
    writer: UnixStream,
}

impl Agent {
    /// Connect to the guest port and read its first Hello. A refused
    /// connection, a dead stream, or an end before the Hello is retried
    /// until `TIMEOUT`, and each retry is logged. Returns the agent and the
    /// number of retries.
    fn open(vm: &Vm, since: Instant) -> Result<(Self, u32), String> {
        let mut retries = 0;
        loop {
            match Self::try_open(vm) {
                Ok(agent) => return Ok((agent, retries)),
                Err(reason) if since.elapsed() < TIMEOUT => {
                    retries += 1;
                    if reason != "guest not listening" {
                        eprintln!("retry {retries} at {:?}: {reason}", since.elapsed());
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(reason) => return Err(format!("no guest connection: {reason}")),
            }
        }
    }

    fn try_open(vm: &Vm) -> Result<Self, String> {
        let fd = match block_on(vm.connect(PORT)) {
            Ok(fd) => fd,
            Err(Error::GuestNotListening { .. }) => return Err("guest not listening".into()),
            Err(error) => panic!("{error}"),
        };
        let stream = UnixStream::from(fd);
        stream
            .set_read_timeout(Some(Duration::from_secs(60)))
            .map_err(|error| format!("dead stream: {error}"))?;
        let mut agent = Self {
            reader: BufReader::new(stream.try_clone().map_err(|error| error.to_string())?),
            writer: stream,
        };
        let mut line = String::new();
        match agent.reader.read_line(&mut line) {
            Ok(0) => return Err("end before Hello".into()),
            Ok(_) => {}
            Err(error) => return Err(format!("read before Hello: {error}")),
        }
        eprintln!("guest: {}", line.trim_end());
        assert!(matches!(
            serde_json::from_str(&line).unwrap(),
            GuestToAgent::Hello {
                handoff_protocol_version: 2,
                configuration_received: false,
                ..
            }
        ));
        Ok(agent)
    }

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

    /// Configure the runner, answer its job with GitHubOnly, and return the
    /// runner exit.
    fn run_job(&mut self) -> RunnerExit {
        self.send(&AgentToGuest::ConfigureRunner {
            protocol_version: ProtocolVersion::CURRENT,
            runner_id: RunnerId("runner-e2e".into()),
            organization_id: OrganizationId("org-e2e".into()),
            registration: Secret::new("fake-jit-for-the-vm-test"),
        });
        loop {
            match self.receive() {
                GuestToAgent::JobStarting {
                    job_id, runtime, ..
                } => {
                    assert_eq!(job_id, JobId(JOB.into()));
                    assert_eq!(runtime.proxy_url, "http://127.0.0.1:8230");
                    assert_eq!(runtime.docker_data_bytes, 0);
                    assert_eq!(runtime.github_token.expose_secret(), "original-token");
                    self.send(&AgentToGuest::PrepareJob {
                        job_id,
                        decision: JobDecision::GitHubOnly,
                    });
                }
                GuestToAgent::JobPrepared { outcome, .. } => {
                    assert_eq!(outcome, JobPreparation::default());
                }
                GuestToAgent::RunnerExited { exit, .. } => return exit,
                _ => {}
            }
        }
    }
}

/// The fake entry exits with 3 only when each of its checks passes. Then
/// the guest powers the VM off.
fn expect_success_and_power_off(vm: &Vm, address: Ipv4Addr, exit: RunnerExit) {
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
}

/// Configure a runner over vsock, hold its job for a decision, and see the
/// guest report the exit and power the VM off.
#[test]
#[ignore = "starts a macOS VM on a Mac and needs SSH into the guest"]
fn guest_runs_one_runner_over_vsock_and_powers_off() {
    let base = Bundle::new(variable("CIPROXY_VZ_BASE"));
    let clone = Clone::new(&base, "e2e", &Identity::fresh().unwrap());
    let vm = Vm::new(&clone.0, &settings()).unwrap();
    let started = Instant::now();
    block_on(vm.start()).unwrap();
    let address = guest_address(&vm, started);
    start_guest(address);
    let (mut agent, _) = Agent::open(&vm, started).unwrap();
    let exit = agent.run_job();
    expect_success_and_power_off(&vm, address, exit);
}

/// The saved state of a seat, as the agent makes it: read the first Hello,
/// close with no configuration, then pause, save, and stop. A job VM from a
/// clone of that seat restores the state and takes its first configuration
/// on a new connection, through the listener from before the save.
#[test]
#[ignore = "starts two macOS VMs on a Mac, one after the other"]
fn restored_guest_takes_its_first_configuration_after_a_closed_hello() {
    let base = Bundle::new(variable("CIPROXY_VZ_BASE"));
    let identity = Identity::fresh().unwrap();
    let seat = Clone::new(&base, "seat", &identity);
    let state = seat.0.root().join("state.vzvmsave");
    let vm = Vm::new(&seat.0, &settings()).unwrap();
    let started = Instant::now();
    block_on(vm.start()).unwrap();
    start_guest(guest_address(&vm, started));
    let (agent, _) = Agent::open(&vm, started).unwrap();
    drop(agent);
    std::thread::sleep(Duration::from_secs(1));
    block_on(vm.pause()).unwrap();
    block_on(vm.save(&state)).unwrap();
    block_on(vm.stop()).unwrap();
    assert_eq!(block_on(vm.stopped()), StopReason::Host);
    drop(vm);

    let job = Clone::new(&seat.0, "job", &identity);
    let vm = Vm::new(&job.0, &settings()).unwrap();
    let restoring = Instant::now();
    block_on(vm.restore(&job.0.root().join("state.vzvmsave"))).unwrap();
    block_on(vm.resume()).unwrap();
    let opened = Agent::open(&vm, restoring);
    eprintln!("restore to Hello: {:?}", restoring.elapsed());
    let address = guest_address(&vm, restoring);
    let log = ssh(address, "cat guest.log");
    eprintln!("guest log:\n{}", String::from_utf8_lossy(&log.stdout));
    let (mut agent, retries) = opened.unwrap();
    eprintln!("retries before the Hello: {retries}");
    let exit = agent.run_job();
    expect_success_and_power_off(&vm, address, exit);
}
