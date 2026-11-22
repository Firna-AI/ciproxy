# The agent on a Mac

Status: design for workstream A of macOS support. It is not built yet.
Items marked **Check** need a test on the test Mac before the build uses
them.

The agent runs on an Apple Silicon Mac with macOS. It runs each GitHub
Actions job in a fresh macOS VM, through Virtualization.framework and the
`ciproxy-vz` crate. The core of the agent stays the same on Linux and on
macOS: identity, the gateway connection, reports, drain, removal, runner
registrations, and the cache handoff. This page gives the parts that are
different on a Mac.

## Terms

- **VM helper:** a `ciproxy-agent vm-run` process that owns one `ciproxy-vz`
  VM. Each VM helper is its own launchd job.
- **VM seat:** one of the 2 macOS VMs that Apple's licence allows at a time.
  A seat has the index 0 or 1. It is the `network_index` of a `VmRecord`.
  The plan calls it a slot. This page says seat, because a slot in
  `Resources` is a unit of CPU and memory.
- **Seat identity:** the fixed MAC address and the fixed VZ machine
  identifier of a seat. A saved state needs both to restore.
- **Saved state:** a paused VM of one seat, saved to disk with
  `Vm::save`, with the APFS clone of its disk and auxiliary storage.

## Processes

| Process       | launchd label             | Lifetime                    |
| ------------- | ------------------------- | --------------------------- |
| Agent daemon  | `dev.ciproxy.agent`       | Always, restarts on failure |
| VM helper     | `dev.ciproxy.vm.LOCAL_ID` | One VM, from start to stop  |
| Machine cache | `dev.ciproxy.cache`       | Always, see Machine cache   |

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
  secret: the bundle paths, the CPU count, the memory size, the seat, and
  the start kind (cold or restore with the saved-state path).
- The VM helper listens on `area/vms/LOCAL_ID/control.sock`. The socket has
  mode `0600`, and its directory has mode `0700`, both owned by root.
- When the VM stops, the VM helper writes `exit.json` with the
  `ciproxy-vz` `StopReason`, then exits.
- SIGTERM makes the VM helper stop the VM at once. This is the same as a
  `systemctl stop` of a Firecracker unit.

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

The agent keeps one `WAIT` connection open for each VM. Its answer, or its
end, tells the agent that the VM stopped. The backend then sends
`RunnerExited` to the core, as L's `Vms` interface says.

The vsock fd passes to the agent, so the runner registration never passes
through the VM helper. V checks on the Mac that a `ciproxy-vz` vsock fd
stays valid after `SCM_RIGHTS` and after the framework releases its
connection object.

## Host-guest transport

Agreed with G through the parent:

- `ciproxy-guest` listens on vsock port 5252 in the guest. The agent
  connects. The frames are newline-delimited JSON of `AgentToGuest` and
  `GuestToAgent`, with the 1 MiB limit on the agent side.
- The registration is only in `ConfigureRunner`. The end is
  `RunnerExited`. The guest replays its state after a reconnect.
- On macOS, `GuestTransport::connect` sends `CONNECT 5252` to the control
  socket and takes the fd. A refused connect
  (`GuestNotListening`) is retryable. The agent retries until the guest
  listens. The rest of `guest_transport.rs` stays the same.
- When the runner ends, the guest runs `shutdown -h now`.
- The guest has no Docker. The agent offers no Docker disk on macOS.

So the registration is never in a process argument, a file, launchd
data, or a log. This is the same rule as on Linux.

## Paths

| Path                                             | Use                        |
| ------------------------------------------------ | -------------------------- |
| `/usr/local/bin/ciproxy-agent`                   | The signed agent binary    |
| `/Library/LaunchDaemons/dev.ciproxy.agent.plist` | The agent daemon           |
| `/etc/ciproxy-agent/environment`                 | Settings, mode `0600`      |
| `/var/db/ciproxy`                                | Control state, mode `0700` |
| `/var/db/ciproxy/state.json`                     | Identity and VMs, `0600`   |
| `/var/db/ciproxy/vms/`                           | VM helper launchd files    |
| `/var/db/ciproxy/area`                           | Mount of the area volume   |
| `/var/log/ciproxy/agent.log`                     | Agent and VM helper logs   |
| `/etc/newsyslog.d/ciproxy.conf`                  | Log rotation               |

`/var/db` is where macOS keeps daemon data. Its path has no space, as
`/var/lib/ciproxy` on Linux. Install creates `/usr/local/bin` with mode
`0755` when a fresh Mac does not have it.

launchd has no environment file. On macOS, `run` reads
`/etc/ciproxy-agent/environment` itself, with the same keys as Linux.

## The launchd daemon

`/Library/LaunchDaemons/dev.ciproxy.agent.plist` sets these keys:
`ProgramArguments` `/usr/local/bin/ciproxy-agent run`, `RunAtLoad`,
`KeepAlive` with `SuccessfulExit` false, `ThrottleInterval` 5, and the log
path for standard output and standard error.

- `KeepAlive` with `SuccessfulExit` false restarts the agent after a
  failure, as `Restart=on-failure` does. A removed machine exits with 0
  after its jobs, so launchd does not start it again.
- Install runs `launchctl bootstrap system FILE`, then
  `launchctl enable system/dev.ciproxy.agent`. A reinstall runs
  `launchctl kickstart -k system/dev.ciproxy.agent`.
- Stop runs `launchctl bootout system/dev.ciproxy.agent`. VM helpers are
  other launchd jobs, so they keep running.
- The agent holds a `PreventUserIdleSystemSleep` power assertion while it
  runs. A sleeping Mac takes no jobs, and its VMs stop. The agent does not
  change the power settings of the Mac.

## Storage

The agent keeps images, VM bundles, saved states, and the machine cache on
one APFS volume of its own, named `ciproxy`, in the container of the data
volume. The volume mounts at `/var/db/ciproxy/area`.

- The volume has a quota. So jobs and images cannot fill the disk of the
  Mac. This replaces the XFS file of Linux.
- APFS clones work inside the volume. A clone takes 1 ms and no space at
  first.
- Install turns off Spotlight (`mdutil -i off`) and Time Machine
  (`tmutil addexclusion`) on the volume.
- `area.json` records the volume UUID. Install and uninstall act only on
  the volume with that UUID and the name `ciproxy`.
- **Check:** `diskutil apfs addVolume` with `-quota` and `-mountpoint`, and
  an `/etc/fstab` line by UUID, mount the volume at boot before the agent.
  If not, the agent waits for the mount and reports zero seats.

| Path in the area   | Use                                    |
| ------------------ | -------------------------------------- |
| `images/HASH/`     | Checked image bundles and downloads    |
| `vms/LOCAL_ID/`    | The VM bundle clone, `vm.json`, socket |
| `seats/N/current/` | The saved state of seat N              |
| `seats/N/build/`   | A saved state while it is built        |
| `cache/data`       | Machine cache data                     |

The image bundle holds the disk, the auxiliary storage, the hardware
model, and the image metadata. I owns its format. The agent checks the
size and SHA-256 before it publishes an image, as on Linux. The disk size
of a job is the free space in the image disk. No root growth happens on
macOS in the first version.

## VM seats and saved states

The agent creates the 2 seat identities at install and keeps them in
`state.json`. Each MAC address is a locally administered address.
`allocate` gives the lowest free seat. The agent counts each VM helper,
also a saved-state build, against the limit of 2.

A seat has a valid saved state when its `saved.json` names the newest
image, the seat identity, and a save time less than 12 hours ago.

Build of a saved state, only when the seat is free and the machine is not
draining:

1. Clone the image bundle with the seat identity into `seats/N/build/`.
2. Cold-start it in a VM helper. Wait for the guest `Hello`.
3. Close the vsock connection. Pause and save the VM. Stop it.
4. Write `saved.json`. Rename `build` to `current`.

A start for a job on a seat:

1. If a build runs on the seat, stop it and delete `build`.
2. With a valid saved state, clone `current` into `vms/LOCAL_ID/` and
   restore from its state file. Else clone the image and boot cold.
3. Connect to port 5252, then deliver `ConfigureRunner`.

One saved state restores one VM at a time. A seat has its own saved state,
so the 2 seats restore together. The agent uses the state again for the
next job of the seat. **Check:** a restore works with the disk of a fresh
APFS clone of the saved disk at another path.

### Clock and address after a restore

A restored guest has the clock of the save time. TLS and the runner need
the correct time before the runner starts. **Check:** the guest clock after
a restore from a state that is 1 hour old.

If the guest clock is wrong, I propose to P and G: the agent sends its UTC
time in each new connection, and the guest sets its clock before it starts
the runner. The 12-hour limit keeps the step small.

The guest keeps its DHCP address after a restore. The seat MAC is fixed,
so the vmnet DHCP server gives the same address again. The 12-hour limit
is less than the lease time. The agent uses vsock, not the guest address.
It reads the address with `lease::lookup` only for logs. **Check:** the
lease time in `/var/db/dhcpd_leases` and the address after an old restore.

## Network isolation

The spike proved these pf rules. Without them, a guest reaches the host,
the LAN, and the router.

- The agent loads its rules into the anchor `com.apple/ciproxy`.
  The main pf rules of macOS already load every `com.apple/*` anchor.
- On guest traffic in on the vmnet bridge, the rules pass DHCP to the
  host, DNS to the host, and TCP to the machine cache port on the host.
- They block all other traffic to the host, to 10/8, 172.16/12,
  192.168/16, 169.254/16, and 100.64/10, from guest to guest, and all IPv6.
- They also block TCP to the machine cache port from all interfaces other
  than `lo0` and the vmnet bridge.
- The IPv4 rules also match the vmnet subnet as the source, with no
  interface. The agent reads the subnet from the vmnet settings.
- The agent turns pf on with `pfctl -E` and keeps the token in
  `state.json`. Uninstall gives the token back with `pfctl -X`.

The rules stay in the kernel while the agent restarts. A reboot removes
them. So the agent loads them at each start. Before each VM start or
restore, it checks that pf is on and that the anchor holds the expected
rules. If not, it loads them again. If that fails, the start fails.
Each report cycle runs the same check. When it fails with VMs running, the
agent stops those VMs, because their isolation is gone.

After a VM starts, the agent checks that the VM's interface is a member of
the bridge in the rules (`bridge100` in the spike). If not, it stops the
VM. **Check:** the bridge name stays `bridge100` when another vmnet user
runs.

Uninstall flushes the anchor with `pfctl -a com.apple/ciproxy -F all`.

## Machine cache

G builds the cache sidecar for macOS. The agent runs it as the launchd
daemon `dev.ciproxy.cache` with a hidden system account `_ciproxycache`,
on `area/cache/data`, as on Linux. The guest reaches it at
`http://GATEWAY:7887`, where `GATEWAY` is the host address on the vmnet
bridge. Until the cache runs, the agent reports the cache as unavailable,
and jobs use the GitHub cache through the existing fallback.

## Hardware report

| Field            | Source                                |
| ---------------- | ------------------------------------- |
| `platform`       | `macos_arm64`                         |
| `processor`      | `sysctl machdep.cpu.brand_string`     |
| `threads`        | `sysctl hw.logicalcpu`                |
| `memory_bytes`   | `sysctl hw.memsize`                   |
| `virtualization` | See below                             |
| `disks`          | The area volume: quota and free bytes |

The agent reads the values with `sysctlbyname`, with no subprocess.
`virtualization` is:

- `framework_unsupported` when `VZVirtualMachine.isSupported` is false, or
  the hardware model of the newest image is not supported.
- `operating_system_too_old` when the macOS of the Mac is older than the
  minimum host version of the newest image.
- `entitlement_missing` when the binary does not have
  `com.apple.security.virtualization`.
- `available` otherwise.

Seats follow `Slots::for_machine` with P's rule, then
`Platform::vm_limit()`. So a Mac reports at most 2.

## Install

1. Refuse a user other than root, a processor other than arm64, and a
   macOS older than the minimum. The message names the minimum, the
   current version, and the repair.
2. Create `/usr/local/bin` when it is missing. Copy the binary to a staging
   name, then rename it to `/usr/local/bin/ciproxy-agent`. A rename gives
   a new file. macOS caches a code signature for each file, so a write
   into a running signed binary makes the kernel stop it.
3. Create or check the area volume. Load the pf rules.
4. Register, write the launchd file, and start the daemon.

The real minimum is the macOS of the image, because Virtualization.framework
does not run a guest newer than the host. The image comes after
registration. So install checks the minimum of the agent build. Then the
report gives `operating_system_too_old` when a new image needs more.

## Drain, update, and uninstall

- Drain keeps the core rules. Also, the agent starts no saved-state build,
  and it stops a running build.
- The hourly update uses `cli-updater` with the target of the build,
  `aarch64-apple-darwin` on a Mac. It installs by rename, then runs
  `launchctl kickstart -k system/dev.ciproxy.agent`. VM helpers keep the
  old binary until their VMs stop.
- Uninstall stops the daemon and waits for the runtime lock, as on Linux.
  It stops idle VMs and waits for job VMs. Then it stops builds, flushes
  the pf anchor, gives back the pf token, deletes the area volume by UUID,
  and removes the launchd files, the binary, the logs, and the state.

## Recovery after an agent restart

At start, the agent compares `state.json`, the launchd jobs with the
label prefix `dev.ciproxy.vm.`, and the area.

- A recorded VM with a running VM helper: the agent connects to its control
  socket and calls `reattach`. The guest replays its state.
- A recorded VM with no VM helper: the VM stopped. The agent reads
  `exit.json`, sends `RunnerExited`, and removes the bundle and the
  launchd file.
- A VM helper with no record: the agent stops it and removes its files.
- A `vms/` folder with no record and no VM helper: the agent removes it.
- A `seats/N/build/` folder: the agent removes it. A `current` folder that
  fails its `saved.json` check: the agent removes it.
- After a reboot, no VM helper runs. So every recorded VM stops as above.
  With FileVault on, macOS starts no daemon until a person logs in once.

A start that fails with `VirtualMachineLimit` means that another program
runs a macOS VM. The agent reports a start failure and logs the reason.
