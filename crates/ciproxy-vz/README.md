# ciproxy-vz

Run macOS VMs on Apple Silicon over Apple's Virtualization.framework.
The agent on a Mac uses this crate to run each macOS job in a fresh VM.
The image build uses it to install macOS and to drive the Setup Assistant.
This crate has the Apache-2.0 licence in [LICENSE](LICENSE).

## Responsibilities

Give a safe Rust API over the parts of Virtualization.framework that
ciproxy needs: install, bundle files, configuration, APFS clones, start,
pause, save, restore, stop, vsock, and VNC.
Keep the pure parts testable on Linux.
Use no workspace crate. The bindings come from `objc2-virtualization`.
Hold no ciproxy policy, such as seats, labels, or pf rules. The agent
holds that policy.

## What This Crate Does

The pure parts build on every platform:

- `MacAddress` reads and writes the normal text form, such as
  `a2:4c:08:01:00:0f`, and the lease form without leading zeros, such as
  `a2:4c:8:1:0:f`. `random_local()` makes a locally administered address.
- `lease::lookup(LEASE_FILE, mac)` finds the IPv4 address of a VM in
  `/var/db/dhcpd_leases`. The newest lease of the address wins. A missing
  file means no lease yet.
- `Bundle` names the files of one VM directory and reads and writes them:

  | File | Content |
  | --- | --- |
  | `disk.img` | The raw, sparse startup disk |
  | `aux.img` | The auxiliary storage of the Mac platform |
  | `hardware-model.bin` | `VZMacHardwareModel.dataRepresentation` |
  | `machine-identifier.bin` | `VZMacMachineIdentifier.dataRepresentation` |
  | `mac-address.txt` | The MAC address of the NAT device |

- `Identity` is the MAC address and the machine identifier of a VM. A saved
  state restores only into a VM with the same identity.
- `VmSettings` gives the CPU count, the memory size, and an optional
  display. `check(limits)` rejects values outside the host and guest
  limits.

The macOS parts build only for `target_os = "macos"`:

- `RestoreImage::latest()` asks Apple's catalogue for the newest supported
  `.ipsw`. The caller downloads it. `RestoreImage::load(path)` loads a
  local file, and `requirements()` gives the hardware model and minimums.
- `install(bundle, image, settings, disk_bytes, progress)` creates a new
  bundle and installs macOS into it.
- `clone_bundle(source, target, identity)` clones a bundle with
  `clonefile(2)` and writes the given identity. Use `Identity::fresh()` for
  a VM without a saved state.
- `Vm::new(bundle, settings)` makes a VM with one NAT virtio network device
  with the bundle's MAC address, one virtio block disk, entropy, a memory
  balloon, one virtio socket device, a keyboard, and a trackpad.
- `Vm` has `start`, `start_recovery`, `stop`, `request_stop`, `pause`,
  `resume`, `save`, `restore`, and `state`.
- `Vm::stopped()` resolves when the current run stops: `StopReason::Guest`
  after `shutdown -h now` in the guest, `Host` after `stop`, or `Error`.
- `Vm::connect(port)` opens a vsock stream to the guest. It returns a
  duplicate `OwnedFd` with `FD_CLOEXEC`, so the stream outlives the
  framework's connection object and can pass to another process with
  `SCM_RIGHTS`. A guest that does not listen yet gives
  `Error::GuestNotListening`, which a caller retries.
- A start beyond the licence limit of 2 active macOS VMs gives
  `Error::VirtualMachineLimit`. It is `VZErrorVirtualMachineLimitExceeded`.
  A paused VM still counts. After the error, the VM is in `VmState::Error`.
- `stop` does nothing, and succeeds, for a VM in `VmState::Stopped` or
  `VmState::Error`, so a caller can always stop a VM that it gives up.

The futures need no particular runtime, and they are `Send`.

### Threading

No part needs AppKit, `NSApplication`, or the main thread. Each `Vm` makes
a private serial dispatch queue and creates its `VZVirtualMachine` with
`initWithConfiguration:queue:`. Every call to the VM runs on that queue, and
the framework calls the completion blocks and the delegate there.

A VM lives inside the process that made it. It stops when the last `Vm`
handle drops or when the process exits.

### User context

A LaunchDaemon, as root or with `UserName`, can start, run, and stop VMs.
Only a process in the login session
of a normal user can save and restore a VM: a LaunchAgent in the
`gui/<uid>` domain of a user who is logged in at the console. Workstream A
tested its VM helper on the test Mac, with macOS 26.7.3 on the host:

| Process | Start and run | Save and restore |
| --- | --- | --- |
| `gui/<uid>`, the user's login session | Pass | Pass |
| LaunchDaemon with `UserName` | Pass | `save` stops the VM |
| LaunchDaemon with `UserName` and `SessionCreate` | Pass | `save` stops the VM |
| LaunchDaemon as root | Pass | `save` stops the VM |
| Root under `sudo` | Fails | Not reached |
| `user/<uid>` background domain | `launchctl bootstrap` fails | Not reached |

- In `gui/<uid>`, the save took 3 s and wrote 1.7 GB for 8 GB of memory.
  The state restored into an APFS clone at another path and ran 6 s later.
  A vsock connect after the restore passed.
- In a LaunchDaemon, also with `UserName` and with `SessionCreate`, `pause`
  and then `save` stop the VM with `VZErrorDomain` error 1, "Internal
  Virtualization error". The vsock connect passes there.
- Under `sudo`, `start` fails with `VZErrorDomain` error -9, "Unable to
  access security information".
- In `user/<uid>`, `launchctl bootstrap` fails with error 5, "Input/output
  error".

Apple documents only that a saved state is encrypted with a key that is
tied to the host. The likely cause of the failures is that the framework
needs the security session of a logged-in user. This is not proven.

### Private VNC

The `private-vnc` feature adds `Vm::start_vnc(port, password)` over the
private `_VZVNCServer`. Apple does not document it, and it can change in any
macOS release. It has no bind address, so it listens on every interface.
Block its port with pf, and use a random password. `None` uses no password.
Port 0 picks a free port, which `VncServer::port()` gives.

### Signing

A process that runs a VM needs the `com.apple.security.virtualization`
entitlement. An ad-hoc signature is enough for development:

```sh
codesign --force --sign - --entitlements crates/ciproxy-vz/vz.entitlements <binary>
```

## Tests

The pure tests run on any host:

```sh
cargo test --package ciproxy-vz
```

The gate phase `rust-macos` checks and lints this crate for
`aarch64-apple-darwin` on Linux. It needs no Apple SDK.

### Tests on a Mac

The tests in `tests/macos.rs` start real macOS VMs, so they are ignored by
default. They need an Apple Silicon Mac with macOS 14 or later and an
installed bundle with the files above. The guest must give SSH with a key
to its user, and `sudo -n` without a password. Run them one at a time,
because macOS runs at most 2 macOS VMs.

| Variable | Value |
| --- | --- |
| `CIPROXY_VZ_BASE` | The installed bundle. The tests only clone it. |
| `CIPROXY_VZ_WORK` | A directory on the same APFS volume for the clones |
| `CIPROXY_VZ_GUEST_USER` | The SSH user in the guest |
| `CIPROXY_VZ_GUEST_ECHO` | The built `guest_vsock_echo` example |

```sh
cargo build --package ciproxy-vz --example guest_vsock_echo
export CIPROXY_VZ_GUEST_ECHO=$PWD/target/debug/examples/guest_vsock_echo
export CARGO_TARGET_AARCH64_APPLE_DARWIN_RUNNER=crates/ciproxy-vz/sign-and-run.sh
cargo test --package ciproxy-vz --test macos -- --ignored --test-threads=1 --nocapture
```

The runner script signs each test binary with the entitlement, then runs
it. The tests print the times of the clone, the boot, the save, and the
restore.

## Key Code

- [src/lease.rs](src/lease.rs) parses the DHCP leases.
- [src/bundle.rs](src/bundle.rs) owns the bundle files.
- [src/macos/vm.rs](src/macos/vm.rs) owns the VM queue and its operations.
- [src/macos/configuration.rs](src/macos/configuration.rs) builds the
  configuration.
- [src/macos/delegate.rs](src/macos/delegate.rs) reports stops.
- [src/macos/vsock.rs](src/macos/vsock.rs) opens guest streams.
- [src/macos/vnc.rs](src/macos/vnc.rs) wraps the private VNC server.
