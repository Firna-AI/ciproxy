# The agent on a Mac

The agent runs on an Apple Silicon Mac with macOS. It runs each GitHub
Actions job in a fresh macOS VM, through Virtualization.framework and the
`ciproxy-vz` crate. The core of the agent stays the same on Linux and on
macOS: identity, the gateway connection, reports, drain, removal, runner
registrations, and the cache handoff. This page gives the parts that are
different on a Mac.

Items marked **Check** still need a run on a real Mac.
[Open work](#open-work) lists the parts that are not built yet.

## Terms

- **VM helper:** a `ciproxy-agent vm-run LOCAL_ID` process that owns one
  `ciproxy-vz` VM. Each VM helper is its own launchd job.
- **VM seat:** one of the 2 macOS VMs that Apple's licence allows at a time.
  A seat has the index 0 or 1. It is the `network_index` of a `VmRecord`.
  The plan calls it a slot. This page says seat, because a slot in
  `Resources` is a unit of CPU and memory.
- **Seat identity:** the fixed MAC address and the fixed VZ machine
  identifier of a seat.
- **Saved state:** a paused VM of one seat, saved to disk with
  `Vm::save`, with the APFS clone of its disk and auxiliary storage.

## Code

The parts with no macOS-only dependency live in [`src/macos/`](src/macos/).
They build on macOS and in the tests of every target, so the Linux gate
tests them. The parts that use `ciproxy-vz`, `launchctl`, `pfctl`, or
`diskutil` live in [`src/platform/macos/`](src/platform/macos.rs) and build
only on macOS. The `rust-macos` gate phase type-checks and lints them.

## Processes

| Process      | launchd label             | Lifetime                    |
| ------------ | ------------------------- | --------------------------- |
| Agent daemon | `dev.ciproxy.agent`       | Always, restarts on failure |
| VM helper    | `dev.ciproxy.vm.LOCAL_ID` | One VM, from start to stop  |

A Virtualization.framework VM lives in the process that creates it.
[Agent](../../docs/protocol/agent.md) says that running VMs survive an agent
restart and an agent update. So the agent does not own VMs. Each VM runs in
its own VM helper, as each Linux VM runs in its own systemd unit.

- The agent writes the VM helper's launchd file to
  `/var/db/ciproxy/vms/LOCAL_ID.plist`, then runs
  `launchctl bootstrap system FILE`. The file is not under
  `/Library/LaunchDaemons`, so a reboot does not start it again.
- The launchd file sets `RunAtLoad`, no `KeepAlive`, and `ExitTimeOut` 30.
- The VM helper reads `area/vms/LOCAL_ID/vm.json`. This file holds no
  secret: the bundle path, the CPU count, the memory size, the seat, and
  the start kind. The helper refuses an inconsistent file.
- The VM helper listens on `area/vms/LOCAL_ID/control.sock`. The socket has
  mode `0600`, and its folder has mode `0700`, both owned by root.
- When the VM stops, the VM helper writes `exit.json` with a rename, then
  exits. SIGTERM makes the VM helper stop the VM at once, as a
  `systemctl stop` of a Firecracker unit does.

### Control socket

One request per connection, as one text line: the version
`CIPROXY-VM 1`, a word, and for `CONNECT` the port. The answer is one line:
`OK`, `OK WORD`, or `ERR CODE`. Only enums build the words, so no line
carries free text or a secret. A new agent must still speak to an older VM
helper after an update, so this protocol stays small.
[`vm_control.rs`](src/macos/vm_control.rs) defines it.

| Request        | Answer                                             |
| -------------- | -------------------------------------------------- |
| `STATE`        | `OK PHASE`, for example `OK running`               |
| `CONNECT PORT` | `OK` with the vsock stream fd by `SCM_RIGHTS`      |
| `STOP`         | `OK` after `request_stop`                          |
| `SAVE`         | `OK` after pause and `Vm::save` into the VM folder |
| `WAIT`         | `OK REASON` when the VM stops                      |

`SAVE` takes no path, so a request cannot make the helper write outside
its VM folder. `ERR guest-not-listening` is the one retryable answer.

The vsock fd passes to the agent, so the runner registration never passes
through the VM helper. V checked on the Mac that a `ciproxy-vz` vsock fd
stays valid after the framework releases its connection object.
**Check:** the fd works in the agent after `SCM_RIGHTS`.
[`macos-helper-smoke.sh`](scripts/macos-helper-smoke.sh) runs this check.

## Host-guest transport

Agreed with G:

- `ciproxy-guest` listens on vsock port 5252 in the guest. The agent
  connects. The frames are newline-delimited JSON of `AgentToGuest` and
  `GuestToAgent`, with the 1 MiB limit on the agent side.
- The registration is only in `ConfigureRunner`. The end is
  `RunnerExited`. The guest replays its state after a reconnect.
- The guest bridge is shared with Linux. On macOS, its transport sends
  `CONNECT 5252` to the control socket and frames the passed stream. A
  refused connect gives `Error::Disconnected`, so the bridge retries after
  one second until the guest listens.
- The bridge asks the helper for its phase before each connect. A stopped
  or absent helper ends the channel: `exit.json` gives the runner exit, and
  no file means an absent VM.
- When the runner ends, the guest runs `shutdown -h now`.
- The guest has no Docker. The agent offers no Docker disk on macOS.

So the registration is never in a process argument, a file, launchd
data, or a log. This is the same rule as on Linux.

## Paths

| Path                                             | Use                        |
| ------------------------------------------------ | -------------------------- |
| `/usr/local/bin/ciproxy-agent`                   | The signed agent binary    |
| `/Library/LaunchDaemons/dev.ciproxy.agent.plist` | The agent daemon           |
| `/var/db/ciproxy`                                | Control state, mode `0700` |
| `/var/db/ciproxy/state.json`                     | Identity and VMs, `0600`   |
| `/var/db/ciproxy/area.json`                      | The area volume UUID       |
| `/var/db/ciproxy/pf-token`                       | The pf reference token     |
| `/var/db/ciproxy/vms/`                           | VM helper launchd files    |
| `/var/db/ciproxy/area`                           | Mount of the area volume   |
| `/etc/ciproxy-agent/environment`                 | Settings, mode `0600`      |
| `/var/log/ciproxy/agent.log`                     | Agent and VM helper logs   |
| `/etc/newsyslog.d/ciproxy.conf`                  | Log rotation               |

`/var/db` is where macOS keeps daemon data. Its path has no space, as
`/var/lib/ciproxy` on Linux. Install creates `/usr/local/bin` with mode
`0755` when a fresh Mac does not have it.

## The launchd daemon

`/Library/LaunchDaemons/dev.ciproxy.agent.plist` runs
`/usr/local/bin/ciproxy-agent run` with `RunAtLoad`, `KeepAlive` with
`SuccessfulExit` false, and `ThrottleInterval` 5. The agent and its VM
helpers write their JSON logs to `agent.log` themselves. Every 5 s a write
checks whether newsyslog has moved the file, and then opens the path again.
launchd sends early output, such as a panic, to the same file.

- `KeepAlive` with `SuccessfulExit` false restarts the agent after a
  failure, as `Restart=on-failure` does. A removed machine exits with 0
  after its jobs, so launchd does not start it again.
- Install runs `launchctl bootstrap system FILE`, then
  `launchctl enable system/dev.ciproxy.agent`. A reinstall runs
  `launchctl kickstart -k system/dev.ciproxy.agent`.
- Stop runs `launchctl bootout system/dev.ciproxy.agent`. VM helpers are
  other launchd jobs, so they keep running.
- launchd has no environment file. So the job runs `/bin/sh`, which
  loads `/etc/ciproxy-agent/environment` with `set -a` when the file
  exists, is owned by root, and is not writable by group or others. Then
  the shell replaces itself with the agent. The keys are the same as on
  Linux.
- The agent runs `caffeinate -i -w PID` while it runs, so the Mac does not
  go into idle sleep. A sleeping Mac takes no jobs, and its VMs pause. The
  agent does not change the power settings of the Mac.

## Storage

The agent keeps images and VM bundles on one APFS volume of its own,
named `ciproxy`, in the container of the data volume.

- The volume has a quota, so jobs and images cannot fill the disk of the
  Mac. This replaces the XFS file of Linux. The default quota is three
  quarters of the free container space, at least 150 GB.
  `CIPROXY_AREA_BYTES` sets another size.
- Install runs `diskutil apfs addVolume CONTAINER APFS ciproxy -quota
BYTESB -nomount`, records the volume UUID in `area.json`, adds the fstab
  line `UUID=UUID /var/db/ciproxy/area apfs rw,nobrowse`, and mounts the
  volume there. It turns off Spotlight (`mdutil -i off`) and Time Machine
  (`tmutil addexclusion -v`) on the volume.
- Daemon start mounts the recorded volume when it is not mounted. Install
  and uninstall act only on the volume with the recorded UUID and the name
  `ciproxy`.
- APFS clones work inside the volume. A clone takes 1 ms and no space at
  first.
- **Check:** the quota syntax, and that the fstab line mounts the volume at
  boot before the agent starts.

| Path in the area      | Use                                         |
| --------------------- | ------------------------------------------- |
| `images/index.json`   | The published images and the newest one     |
| `images/NAME/bundle/` | A checked image bundle                      |
| `vms/LOCAL_ID/`       | The VM bundle clone, `vm.json`, socket      |
| `seats/N/identity/`   | The seat identity, in the `ciproxy-vz` form |

`NAME` is the hex SHA-256 of the image ID, so an ID never names a path.

### Image store

I owns the image format. One `macos.tar` holds `bundle/disk.img.zst`,
`bundle/aux.img`, `bundle/hardware-model.bin`, `manifest.json`, three other
metadata files, and `macos-content.sha256`.

1. The download streams to `images/NAME.tar`. The agent checks its byte
   count and SHA-256 against the `ImageDownload`.
2. Extraction refuses links, other paths, and duplicate members. It
   decompresses the disk and writes each all-zero 1 MiB block as a hole. It
   hashes each file as it writes it, so the content check needs no second
   read of the disk. The disk must have the released root length.
3. The agent checks `macos-content.sha256` and reads `minimum_host_macos`
   and `free_job_bytes` from the manifest.
4. A rename publishes `images/NAME/`. The index names it the newest image.
   Each image that is neither the newest nor held is removed.

### Job disk

Cal's decision 17 is open. P and A recommend option (b): the image disk
has a fixed size of 200 GB, and each VM holds the image's measured
`free_job_bytes` in the disk accounting. So the agent passes no disk size,
and one saved state per seat serves every label. The agent keeps
`free_job_bytes` in its image index. The core uses it once P adds the field
to the protocol.

## VM seats and saved states

`allocate` gives the lowest free seat. Each seat keeps a fixed identity in
`seats/N/identity/`, made at its first use. A fresh MAC address for each
job would use up the vmnet address pool, because a lease lasts up to one
day on the host.

Each job VM clones the newest image bundle with its seat identity into
`vms/LOCAL_ID/bundle/` and boots cold.

### Saved states are off

Saved states for warm starts are built in the code, but the constant
`SAVED_STATES` in [`platform/macos.rs`](src/platform/macos.rs) turns them
off. On the test Mac, a save worked only in a logged-in user's GUI session
(`gui/UID`). The save stopped the VM in these setups:

- a root job in the system domain;
- a system job with `UserName`;
- a system job with `UserName` and `SessionCreate`.

The user's background domain (`user/UID`) refused the job. A warm start
saves about 3 to 8 s per job. Warm starts in a login session need
auto-login, which macOS does not allow with FileVault on. Cal decides
between cold starts only and an opt-in warm start in a login session.

When the constant is on, the design is as follows. A seat's saved state
restores only for its own seat, image, MAC address, CPU count, memory
size, and agent version, and only for 12 hours.
[`seats.rs`](src/macos/seats.rs) holds this rule, and
[`seat_states.rs`](src/platform/macos/seat_states.rs) builds the states.

1. After a job VM ends, the daemon builds a saved state for its seat with
   the image and the VM settings of that job, when the seat has no usable
   one. Install and uninstall build none.
2. A build clones the image into `seats/N/build/bundle/` and cold-starts it
   in a VM helper. It waits for the guest's first `Hello` on vsock, closes
   the connection with nothing sent, and waits 1 s, so the guest is back in
   `accept`. Then `SAVE` pauses and saves the VM, and the helper stops. A
   failed save fails the build, and nothing is published. The state file
   goes into the bundle, then `saved.json`, and a rename publishes
   `seats/N/current/`.
3. A start cancels a running build on its seat. With a usable saved state,
   it clones `current/bundle`, with the state file in it, and the helper
   restores and resumes it. Else it clones the image and boots cold.
4. When a restore fails, the helper boots the same clone cold in a new VM
   object and writes `restore-failed`. When that VM ends, the agent
   discards the seat's saved state.
5. Daemon start and uninstall stop build helpers, which have no record,
   and remove `seats/N/build/`.

In a `gui/UID` session on the test Mac, the smoke script saved a VM in 3 s
to a 1.7 GB state file, and restored it from an APFS clone at another path.
The guest stream was ready 6 s after the restore began, against 9 s after a
cold start. After a restore, the first stream can be dead, so the guest
bridge and the build retry it.

Facts from the test Mac: a restore from a 12-hour-old state gave the guest
the host's UTC time within one second, because network time is on in the
guest. So no clock message is needed. The guest kept its DHCP address, and
the host renewed the lease of its MAC address at once. The guest lease
time is 3600 s. Cold start to SSH took 7.95 s, save 1.72 s, and restore to
SSH 2.36 s. A paused VM still counts toward the limit of 2.

## Network isolation

The spike proved these pf rules. Without them, a guest reaches the host,
the LAN, and the router. [`pf.rs`](src/macos/pf.rs) builds them.

- The agent loads its rules into the anchor `com.apple/ciproxy`.
  The main pf rules of macOS already load every `com.apple/*` anchor.
- On guest traffic in on the vmnet bridge, the rules pass DHCP to the
  broadcast address and the gateway, and DNS to the gateway.
- They block all other traffic to the host, to 10/8, 172.16/12,
  192.168/16, 169.254/16, 100.64/10, 127/8, multicast, and reserved
  addresses, guest-to-guest traffic, and all IPv6 on the bridge.
- They block TCP to the machine cache port, 7887, on every interface other
  than `lo0`. A pass from the guests to the gateway on that port is added
  when the machine cache runs on macOS.
- The IPv4 rules match the vmnet subnet as the source, with no interface.
  The agent reads the subnet from the vmnet settings, else it uses
  192.168.64.0/24.
- The agent turns pf on with `pfctl -E` when pf is off, and keeps the
  token in `pf-token`. Uninstall gives the token back with `pfctl -X`.

The rules stay in the kernel while the agent restarts. A reboot removes
them. Before each VM start, the agent checks that pf is on and that the
anchor shows the same rules as just after its own load. If not, it loads
them again. If that fails, the start fails. Install loads them too.
Uninstall flushes the anchor with `pfctl -a com.apple/ciproxy -F all`.

While VMs run, a guard checks every 10 s:

- that pf is on and the rules are loaded, and it loads them again when
  they are missing or changed;
- that each guest's DHCP address is inside the subnet that the rules cover.
  The IPv4 rules match that subnet, so a guest on another vmnet network
  would escape them.

When a check fails, the guard stops the affected VMs, because their
isolation is gone. [`guard.rs`](src/platform/macos/guard.rs) holds it.

## Machine cache

The machine cache does not run on macOS yet. `NoMachineCache` gives no
cache session, so every job uses the GitHub cache through the existing
GitHub-only decision. The report says that the cache is unavailable. The
pf pass for the cache port and a launchd daemon for the cache sidecar come
with the macOS cache sidecar of workstream G.

## Hardware report

| Field            | Source                             |
| ---------------- | ---------------------------------- |
| `platform`       | `macos_arm64`                      |
| `processor`      | `machdep.cpu.brand_string`         |
| `threads`        | `hw.logicalcpu`                    |
| `memory_bytes`   | `hw.memsize`                       |
| `virtualization` | See below                          |
| `disks`          | The area volume: capacity and free |

The agent reads the values with `sysctlbyname`. `virtualization` is the
first of:

- `framework_unsupported` when `kern.hv_support` is not 1. `ciproxy-vz` has
  no call for `VZVirtualMachine.isSupported` yet.
- `entitlement_missing` when `codesign` shows no
  `com.apple.security.virtualization` for the running binary.
- `operating_system_too_old` when the Mac's macOS is older than 26.7.2 or
  than the `minimum_host_macos` of the newest image.
- `available` otherwise.

Seats follow `Slots::for_machine` with P's rule, then
`Platform::vm_limit()`. So a Mac reports at most 2.

## Install

1. Refuse a user other than root, a processor other than arm64, and a
   macOS older than 26.7.2, the macOS of the planned image. The message
   names the minimum, the current version, and the repair.
2. Create the control folder, the area volume, and the area folders. Load
   the pf rules.
3. Register. Create `/usr/local/bin` when it is missing. Refuse when
   `/usr/local` or `/usr/local/bin` is not owned by root or is writable by
   group or others, because a user could then replace the binary that the
   root daemon runs. Copy the binary to a staging name, give it root:wheel
   and mode 0755, then rename it into place. `fs::copy` clones the file on
   macOS and keeps the source owner, so the owner change is needed. A
   rename gives a new file. macOS caches a code signature for each file,
   so a write into a signed binary makes the kernel stop it.
4. Write the log rotation and the launchd file, then load or restart the
   daemon.

## Drain, update, and uninstall

- Drain keeps the core rules.
- The hourly update uses `cli-updater` with the target
  `aarch64-apple-darwin`. The updater moves the old binary away and writes
  the new one as a new file. Then the agent runs
  `launchctl kickstart -k system/dev.ciproxy.agent`. VM helpers keep the
  old binary until their VMs stop.
- Uninstall stops the daemon and waits for the runtime lock, as on Linux.
  It stops idle VMs and waits for job VMs. Then it flushes the pf anchor,
  gives back the pf token, deletes the area volume by UUID with its fstab
  line, and removes the launchd files, the binary, the logs, and the state.

## Recovery after an agent restart

- A recorded VM with a running VM helper: the core calls `reattach`. The
  bridge connects again, and the guest replays its state.
- A recorded VM whose helper ended: the bridge reads `exit.json` and sends
  `RunnerExited`. The core then stops the VM, which removes the helper's
  launchd file and folder.
- Daemon start stops each loaded helper with no record and removes each VM
  launchd file and folder with no record. Unreadable state keeps them all.
- After a reboot, no VM helper runs, so every recorded VM ends as above.
  With FileVault on, macOS starts no daemon until a person logs in once.

A start that fails with `VirtualMachineLimit` means that another program
runs a macOS VM. The start fails, and the core reports it.

## Open work

- Warm starts: Cal's decision on saved states (see above). With the
  constant on, the agent also builds a saved state while the machine
  drains.
- **Check** on a real install: the `diskutil` quota syntax, the volume
  mount at boot, and the `pfctl` output that the agent reads.
- The machine cache on macOS.
