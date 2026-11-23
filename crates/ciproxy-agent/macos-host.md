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
tests them. They can use the pure types of `ciproxy-vz`, such as
`VmSettings`, because the tests have `ciproxy-vz` as a dev dependency.
`VmSpec::settings` in [`src/macos/vm_spec.rs`](src/macos/vm_spec.rs) builds
the settings of each VM there. The parts that start VMs or use
`launchctl`, `pfctl`, or `diskutil` live in
[`src/platform/macos/`](src/platform/macos.rs) and build only on macOS. The `rust-macos` gate phase type-checks and lints them.

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
- The memory size in `vm.json` is the label size in decimal bytes, such as
  8,000,000,000 for 8 GB. Virtualization.framework needs whole MiB, so the
  helper rounds it down to a whole MiB. The VM never uses more memory than
  the label and its slots reserve.
- The VM helper listens on `area/vms/LOCAL_ID/control.sock`. The socket has
  mode `0600`, and its folder has mode `0700`, both owned by root.
- `launchctl bootstrap` returns before the helper binds this socket. So a
  start waits up to 60 s until the helper answers on it, and only then
  attaches the guest bridge. The bridge reads a missing socket with no
  `exit.json` as a VM that is gone. A helper that already wrote
  `exit.json` fails the start at once.
- `launchctl bootout` can return before the helper process ends. So a stop
  waits up to 40 s until the helper no longer answers on its socket, and
  only then frees the seat. A new VM on that seat has the same MAC address
  and machine identifier, so it must not start while the old VM runs.
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
named `ciproxy`. By default, the volume is in the container of the data
volume. `CIPROXY_AREA_DISK` puts it on another APFS disk, as
[Area on another disk](#area-on-another-disk) says.

- The volume has a quota, so jobs and images cannot fill the disk of the
  Mac. This replaces the XFS file of Linux. The default quota is three
  quarters of the free container space, at least 150 GB.
  `CIPROXY_AREA_BYTES` sets another size.
- Install runs `diskutil apfs addVolume CONTAINER APFS ciproxy -quota
BYTESB -nomount`, records the volume UUID in `area.json`, adds the fstab
  line `UUID=UUID /var/db/ciproxy/area apfs rw,nobrowse`, and mounts the
  volume there. It writes `.metadata_never_index` at the volume root,
  because Spotlight does not manage a `nobrowse` volume and `mdutil -i off`
  fails there. It excludes the volume from Time Machine
  (`tmutil addexclusion -v`).
- Daemon start mounts the recorded volume when it is not mounted. Install
  and uninstall act only on the volume with the recorded UUID and the name
  `ciproxy`. For a volume on another disk, `area.json` also records the
  UUID of the volume's container, so uninstall deletes the volume only in
  that container.
- Each mount by the agent (daemon start, install, `prepare-host`, and
  `import-image`) checks that the volume uses file owners
  (`GlobalPermissionsEnabled`). macOS can ignore owners on an external
  volume, and then any user could write the root-only files of the agent.
  The agent runs `diskutil enableOwnership`, and refuses a volume where
  owners stay off.
- APFS clones work inside the volume. A clone takes 1 ms and no space at
  first.
- **Check:** the quota syntax, and that the fstab line mounts the volume at
  boot before the agent starts.

| Path in the area               | Use                                         |
| ------------------------------ | ------------------------------------------- |
| `images/index.json`            | The published images and the newest one     |
| `images/NAME/bundle/`          | A checked image bundle                      |
| `images/NAME.tar.partial`      | The bytes of a download so far              |
| `images/NAME.tar.partial.json` | The image ID, SHA-256, and size of them     |
| `vms/LOCAL_ID/`                | The VM bundle clone, `vm.json`, socket      |
| `seats/N/identity/`            | The seat identity, in the `ciproxy-vz` form |

### Area on another disk

A Mac with a small internal disk can keep the area on another APFS disk,
such as an external SSD. Set `CIPROXY_AREA_DISK` for `prepare-host` or the
first install:

```sh
sudo env CIPROXY_AREA_DISK=/Volumes/NAME /usr/local/bin/ciproxy-agent install --token TOKEN
```

- The value is a mounted path, such as `/Volumes/NAME`, or an APFS
  container reference, such as `disk5`. With no value, the agent uses the
  startup disk, as before, and the record keeps its older form.
- The agent reads the container and its free space from the target with
  `diskutil info -plist` and `diskutil apfs list -plist`. It reads the
  disk facts from the container's physical store.
- It refuses a volume that is not APFS, a read-only volume or disk, a
  target that `diskutil` does not know, such as a network share, and a
  disk image, which can be on a network share. Each refusal gives a repair
  line.
- It warns, and goes on, when the disk is external or removable. The
  record keeps this fact, and `prepare-host` prints it.
- The setting applies only to a new volume. With a recorded volume, the
  agent keeps that volume and warns when the setting names another disk.
  To move the area, uninstall first.

#### Full Disk Access for an external disk

macOS privacy control (TCC) blocks a root launchd job from a volume on an
external or removable disk. The mount point under `/var/db` does not
change this, because TCC sorts the volume by its disk. A terminal, or
`sshd`, can have the access that the daemon does not have. So
`prepare-host` and `import-image` from a terminal work, and the daemon
fails. The daemon then needs Full Disk Access:

1. Open **System Settings**, then **Privacy & Security**, then **Full Disk
   Access**.
2. Select **+**, press Command-Shift-G, enter
   `/usr/local/bin/ciproxy-agent`, and turn the entry on.

The agent finds a block as follows:

- At each start, with an area on another disk, the daemon lists the
  volume root and writes and removes one file there. When root gets
  "Operation not permitted" or "Permission denied", the daemon stops with
  the repair line "macOS blocks the agent from the area volume on an
  external disk. Give /usr/local/bin/ciproxy-agent Full Disk Access …".
  launchd starts it again every 5 s, so the agent goes on by itself once
  it has the access.
- The daemon writes the result of the check to
  `/var/db/ciproxy/area-access`, on the startup disk, with the start time
  of its process. Only the launchd job writes it. After `install`, or
  `prepare-host --with-service`, loads the daemon, it waits up to 30 s for
  a result of a daemon process that started after the load. It fails with the same repair line when the daemon is
  blocked. With no result, it goes on and names the log. A failed install
  keeps the registration, and the agent starts once it has the access.
- `prepare-host` with no service only prints that the daemon needs the
  access, because its own check would test the terminal.
- A command from a terminal that checks the area, such as an uninstall
  with recorded VMs, tests the access of the terminal app or `sshd`. Its
  repair line names them.
- A failed write of the check file for another cause, such as a full
  volume, only gives a warning, so a full volume does not stop the start.

TCC keeps the grant for one code identity:

- A binary with an ad-hoc signature, such as a local build, is known by
  its hash. Each new build loses the grant, so grant it again after each
  copy to `/usr/local/bin`.
- A release binary with a Developer ID signature is known by its team and
  its identifier, `dev.ciproxy.agent`. The grant then stays across agent
  updates.
- A Mac under MDM can get the grant from a Privacy Preferences Policy
  Control (PPPC) profile: the service `SystemPolicyAllFiles`, the
  identifier `dev.ciproxy.agent` of type `bundleID`, and the code
  requirement of the signed release.

The disk must stay connected. The agent then acts as follows:

- **At boot, before the disk is there:** daemon start finds no volume with
  the recorded UUID. It stops with the repair line "The ciproxy volume is
  missing. Connect the disk that holds it." launchd starts the daemon again
  every 5 s. When the disk comes, macOS mounts the volume from the fstab
  line, and the next start goes on as normal. A disk that mounts late gives
  the same result. A mount by the agent and by fstab at the same time is
  not an error: the agent checks the mount point again.
- **When the disk is unplugged while the agent runs:** the mount point is
  again a bare folder on the startup disk. The agent sees this from the
  device number of the folder. The hardware report then has no disk, so
  the Mac has no job disk, and the control plane sends it no job. A VM
  start refuses with the same repair line. Within 10 s, the guard stops
  every VM, because their disks are gone, and their jobs fail. When the
  disk is back, macOS mounts the volume again, and the Mac takes jobs
  again with no restart.
- **Uninstall with the disk gone:** with no recorded VM, uninstall goes on.
  It warns that the volume stays on its disk. It removes the bare mount
  folder with any file in it, then the fstab line, and the record last.
  When `diskutil` finds the volume but not its container, uninstall stops
  and keeps the record, so a retry decides. Delete the volume later with
  `diskutil apfs deleteVolume UUID`. With a recorded VM, uninstall needs
  the disk, because it drains the VMs first.

`NAME` is the hex SHA-256 of the image ID, so an ID never names a path.

### Image store

I owns the image format. One `macos.tar` holds `bundle/disk.img.zst`,
`bundle/aux.img`, `bundle/hardware-model.bin`, `manifest.json`, three other
metadata files, and `macos-content.sha256`.

1. The download streams to `images/NAME.tar.partial`. Its sidecar
   `images/NAME.tar.partial.json` holds the image ID, the SHA-256, and the
   byte size of the `ImageDownload`. A broken stream retries with
   `Range: bytes=LENGTH-` three times, 5 s apart, and each new byte resets
   the count. A `5xx` or a `429` retries in the same way. A `429` waits for
   the seconds of its `Retry-After` header instead, at most 60 s. Another
   HTTP error status, such as `403` for an expired URL, ends the command
   and keeps the partial file. A later `FetchImage` with the
   same sidecar values hashes the bytes on disk and continues them. Other
   values, or a file longer than the image, start again from zero, and so
   does a `200` answer to a `Range` request. A `206` from another start, or
   a `416`, empties the file, and the retry asks for the whole archive
   without `Range`. A download removes the partial
   files of other images. One download runs at a time, so a second
   `FetchImage` for the same image waits and then finds the image known.
   After the byte count and the SHA-256 match, the partial file becomes
   `images/NAME.tar`. A mismatch deletes the partial file and its sidecar.
2. Extraction refuses links, other paths, and duplicate members. It
   decompresses the disk and writes each all-zero 1 MiB block as a hole. It
   hashes each file as it writes it, so the content check needs no second
   read of the disk. The disk must have the released root length.
3. The agent checks `macos-content.sha256` and reads `minimum_host_macos`
   and `free_job_bytes` from the manifest.
4. A rename publishes `images/NAME/`. The index names it the newest image.
   Each image that is neither the newest nor held is removed.

#### Image import, a test and operator path

The hidden command `import-image` publishes a local `macos.tar` with no
image host. It is for tests, such as the first end-to-end run, and for an
operator who moves an image by hand. `--help` does not list it.

```sh
sudo /usr/local/bin/ciproxy-agent import-image --archive PATH \
  --image-id ID --sha256 HEX --root-disk-bytes N
```

1. Run it as root after `prepare-host` and before `install`. It needs the
   area volume, and it takes the runtime lock, so it refuses to run while
   the daemon runs. The daemon then loads the image when it starts.
2. It copies the archive to `images/NAME.tar`, owned by root with mode
   `0600`. The copy is an APFS clone when the archive is on the area
   volume, and a full copy when it is not. Stage the archive at the area
   root, such as `/var/db/ciproxy/area/macos.tar`. The command refuses an
   archive in `images/`, because recovery removes each `.tar` file there.
3. It checks the byte count and the SHA-256 of the copy, not of the
   source, so a later change of the source cannot reach extraction.
4. Then it runs the same steps 2 to 4 as a download. A failed check or
   extraction removes the copy and the pending folder. When the command
   is stopped, the next import or daemon start removes them.

An image ID that the index already lists is not imported again.

### Job disk

The image disk has a fixed size, and each VM is an APFS clone of it. A
VM holds its label's free job disk and `MACOS_ROOT_MARGIN_BYTES` (4 GB)
in the disk accounting, through `RootRule` in `src/root_sizing.rs`. The
scaler and the control plane use the same rule. The agent reports the
image's `free_job_bytes` with its image metadata, so a label that asks
for more free disk than the image has fits no Mac with that image. The
agent refuses such a start with a warning. It also refuses a new VM when
the real free space of the area volume is less than the disk that the VM
holds (`real_space_fits` in `src/capacity.rs`), because running clones
can grow past their count.

The image budget is the space of the volume that the job disk does not
use: the job disk is (volume capacity − image budget) × 3/4. The default
budget is 200 GiB. `CIPROXY_IMAGE_BUDGET_BYTES` sets another value, with
the same rule as on Linux: an integer greater than 16,777,216. Each command
reads the setting again, so put it in `/etc/ciproxy-agent/environment` for
the daemon. A Mac records no budget, so a change applies at the next start.
`prepare-host` prints the budget in use.

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
- `operating_system_too_old` when the Mac's macOS is older than 26.7.3 or
  than the `minimum_host_macos` of the newest image.
- `available` otherwise.

Seats follow `Slots::for_machine` with P's rule, then
`Platform::vm_limit()`. So a Mac reports at most 2.

## Install

1. Refuse a user other than root, a processor other than arm64, and a
   macOS older than 26.7.3, the macOS of the macOS image. The message
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
- Daemon start keeps each partial download whose sidecar decodes and whose
  file is not longer than its image. It removes every other partial file
  and sidecar, each checked `images/NAME.tar` that waits for extraction,
  and each `.pending` part.
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
- **Check** on a Mac with an external disk: the `diskutil` keys that the
  area disk checks read, the file-owner check, and the behaviour when the
  disk is missing at boot or unplugged.
- The machine cache on macOS.
