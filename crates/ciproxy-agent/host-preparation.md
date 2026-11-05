# Host preparation

`ciproxy-agent install --token TOKEN` prepares the host before registration.
It also repairs setup before it returns for an existing machine.
It keeps that machine's registration, drain state, and active jobs.
It does not require another install token for an existing registration.

## Distribution and kernel support

Package preparation supports these `/etc/os-release` IDs:

| Distribution | IDs | Package manager |
| --- | --- | --- |
| Debian and Ubuntu | `debian`, `ubuntu` | `apt-get` |
| Fedora and RHEL | `fedora`, `rhel` | `dnf` |
| Rocky Linux and AlmaLinux | `rocky`, `almalinux` | `dnf` |

The adapter selects by ID. It does not assert a distribution version.
The distribution must provide current host tools and the kernel features
below. Other distribution IDs get a named repair message.
Amazon Linux is not included in this adapter.

The host must run x64 Linux as root with systemd and cgroup v2.
Its kernel must support XFS with reflinks and loop devices.
VMs also need KVM, network namespaces, nftables, and vsock.
Missing KVM permits registration and reports zero slots.
Enable virtualization in firmware or in the outer VM to run job VMs.
Install does not change firmware or nested-virtualization settings.

The package manager installs `xfsprogs`, `nftables`, `coreutils`,
`e2fsprogs`, `util-linux`, `kmod`, and `ca-certificates`.
Debian and Ubuntu also get `iproute2` and `procps`.
The DNF distributions get `iproute` and `procps-ng` instead.
APT refreshes package indexes before installation.
Package-manager failures name the component and a repair action.
XFS and loop module failures also give their `modprobe` repair.

## Firecracker and jailer pins

Both executables come from the official
[Firecracker v1.17.0 release](https://github.com/firecracker-microvm/firecracker/releases/tag/v1.17.0).
The archive is `firecracker-v1.17.0-x86_64.tgz`.
The [official release API](https://api.github.com/repos/firecracker-microvm/firecracker/releases/tags/v1.17.0)
supplies its asset SHA-256. The archive also contains `SHA256SUMS`.
The executable values below match the files in that checked archive.

| File | SHA-256 |
| --- | --- |
| Archive | `06094a1108ae9e82aa4c23a775aa92758f53f1175d422270d9d6162cb9ade558` |
| `firecracker-v1.17.0-x86_64` | `99ad0f5cd0514a88aad0e9ae8cfdb3cc3b4ab9d190e1194602406c786b5de7a5` |
| `jailer-v1.17.0-x86_64` | `65ef226e96f0ceda55ba643f445801ef2cc0ea667ef67cad8ac4f406c9c8434f` |

Download and extraction stay in memory with a 32 MiB limit.
The agent checks the archive SHA-256 before it parses members.
It checks each executable SHA-256 before publication.
An atomic rename publishes each checked file from a new staging name.
It never runs rejected bytes.
Each new VM start checks both tool hashes and their ownership.
It fetches a missing or invalid current pair without an install token.
Existing VM restore and stop do not need that pair.
A pending download holds no VM operation lock and does not block reports.
Package preparation runs in install, not on each daemon restart.
The owned tools directory is `/var/lib/ciproxy/tools/v1.17.0`.
It includes a root-only owner marker and the upstream licence notices.
Active VMs keep their existing executable inode during a repair.

## Storage and capacity

Identity and control state stay in `/var/lib/ciproxy` with mode `0700`.
`state.json`, `area.json`, and the backing file have mode `0600`.
The ownership and image locks also stay outside the workload mount.

Every installation uses the same storage form, including on native XFS:

| Path | Purpose |
| --- | --- |
| `/var/lib/ciproxy/area.xfs` | One sparse XFS backing file |
| `/var/lib/ciproxy/area` | Workload mount |
| `area/images` | Verified bases and image downloads |
| `area/jailer` | Independent VM jails and disposable root copies |
| `area/cache` | Prepared directory for machine-cache data |
| `area/docker` | Reserved directory for later sticky Docker disk integration |

The area contains the complete protocol job disk plus its image reserve.
The installed `e2fsprogs` package supplies `resize2fs`. It grows a larger
label's private ext4 root before boot, without changing the base image.
The image budget comes out first, then jobs get three quarters of the rest.
Measured XFS overhead uses only the remaining host reserve.
The [image budget guide](image-budget.md) explains the default setting,
physical write limit, measured sizing, and old-area replacement.
Hardware reports still use the underlying host capacity.
They never apply a second reserve to the mounted loop capacity.
Host growth does not silently increase admission or resize a Ready area.
Free-space health remains bounded by both XFS and parent free space.

The mount unit is `var-lib-ciproxy-area.mount`.
The agent and independent VM units require this mount before they start.
VM units remain independent of the agent service lifetime.
The mount uses `loop,rw,nosuid`. It permits devices and execution for jailer.
It does not use `nodev`, `noexec`, forced unmount, or lazy unmount.

### Combine disks on a machine used only for CI

A machine used only for CI can combine its disks into one filesystem.
Jobs, the machine cache, and Docker disks are disposable.
For example, RAID0 or one LVM volume can combine both disks.
The AX42 calculation uses both 512 GB disks as one 1,024 GB filesystem.
That capacity fits six default jobs after the image budget and host reserve.

Hetzner's `installimage` mirrors two disks by default.
A mirror gives one disk's capacity.
See the [installer's default configuration][installimage-config].

[installimage-config]: https://github.com/hetzneronline/installimage/blob/master/config.sh

## Repair and ownership

Repeated install checks the backing inode, byte count, root ownership,
filesystem kind, XFS UUID, and actual loop backing path.
The host device number can change after a remount.
Each operation uses a fresh device snapshot for replacement checks.
It restores a missing mount unit and enables it for boot.
It mounts an unmounted owned area and repairs missing executable files.
It preserves a mounted area and never restarts that mount for repair.
It does not resize or format a published area.
Fresh Creating areas finish measured growth before they become Ready.
A durable old/target length pair permits interrupted growth to resume.
An interrupted first format can resume only in the unpublished Creating
phase. A matching XFS UUID completes that phase without a second format.

The agent refuses to hide nonempty data under an unmounted mountpoint.
It refuses to adopt an unrecorded backing file or replace a foreign unit.
Restore a damaged backing file and `area.json` together from backup.
Do not create a new ownership record for unknown data.
If creation stopped before an ownership record existed, preserve the
orphan file outside the control directory before another first install.
Never move or replace an area that still has active jobs or other users.

For an old installation with top-level `images` or `jailer` data:

1. Drain the old machine in the dashboard.
2. Wait for its jobs and VMs to finish.
3. Stop the old agent service.
4. Back up `/var/lib/ciproxy`, including `state.json`.
5. Move the stopped top-level `images` and `jailer` directories to that
   backup location outside `/var/lib/ciproxy`.
6. Keep `state.json` in place.
7. Run install again. The new daemon fetches a verified image when needed.

## Uninstall

Uninstall stops the agent service and lets independent VM jobs finish.
Guest detach waits for bridge tasks to release their handles.
Final cleanup checks that every owned VM has finished.
It purges image contents, records removal intent, and stops the mount.
It verifies that the mount is gone before it deletes the backing file.
It preserves the backing file while another loop device still uses it.
It removes the owned mount unit, runtime versions, agent service, and state.
Runtime cleanup uses owner markers and needs no current pin or valid hash.
It preserves unowned runtime directories and files.
An unmount or ownership failure preserves the backing file and identity.
It gives a repair action instead of forcing deletion.
The small control directory and lock files can remain after uninstall.
Distribution packages remain installed because other programs can use them.

## Cache integration

`area/cache` is the machine-cache data location for later composition.
The current agent still uses `/run/ciproxy-cache/control.sock`.
The merged cache sidecar uses HTTP administration instead.
Host preparation does not start that service or claim its HTTP integration
is complete. The sidecar must use this area when that integration is added.
Sticky Docker disk ownership also remains separate integration work.

## Ext4 and KVM smoke

Use a fresh disposable x64 host with `/var/lib` on ext4 and usable KVM.
Use systemd, cgroup v2, and the distribution package manager.
Install `gh`, `jq`, `rg`, and Python 3 for the smoke operator.
The agent installs its own VM and storage tools.
The host needs the documented CPU, memory, and disk minimum.
The two jobs run sequentially, so one slot is sufficient.

The staging gateway, guest, and image must first meet the agent contract
in [the crate guide](README.md#gateway-image-and-cache-integration).
The current merged guest/image/cache differences still block live jobs.
Copy [the workflow fixture](scripts/github-smoke-workflow.yml) to the
private test repository. It includes the `host` mode observation window.
Authenticate `gh` for workflow dispatch and run reads.

Run from the repository root in a root shell:

```sh
export CIPROXY_SMOKE_REPO=owner/private-agent-smoke
export CIPROXY_SMOKE_REF=main
export CIPROXY_AGENT_URL=wss://staging.example/agent/
export CIPROXY_SMOKE_INSTALL_TOKEN='<staging install token>'
export CIPROXY_SMOKE_BINARY=target/debug/ciproxy-agent
export CIPROXY_SMOKE_INTEGRATION_READY=1
bash crates/ciproxy-agent/scripts/smoke-host.sh
```

Use the readiness value only after the staging components are aligned.
The script sets the WSS URL for install and the persistent service.
It installs on ext4, writes a data sentinel, removes the owned mount unit
and jailer, and repeats install to prove repair without data loss.
It then dispatches two held real Docker jobs.
It captures each root disk before normal teardown removes it.
It also repeats install during the first held job and checks the VM PID.
The evidence includes distinct VM IDs and source/clone inode identities.
FIEMAP proves shared physical extents at equal logical offsets in each
root disk and its verified base. Copy flags alone cannot pass the check.
The script then uninstalls and checks removal of the area and mount unit.

Evidence stays under `.context/ciproxy-mvp/host-smoke-*`.
It contains no install token or complete credential state.
Missing external resources report `SKIP` and exit 77.
A failed assertion exits 1. The script never substitutes fake jobs.
