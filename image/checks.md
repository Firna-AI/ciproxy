# Offline image checks

Check the generated QEMU bootstrap without KVM or package downloads:

```sh
python3 image/check-template.py
```

Check the kernel build environment without compiling Linux:

```sh
python3 image/check-kernel-metadata.py
```

This fixture checks neutral metadata at the Make process boundary.
It needs Git, Bash, and Python. It does not prove a real kernel build.

## Kernel build metadata

Kernel builds use `builder@localhost` and a fixed build version.
The timestamp uses the source commit time through
`SOURCE_DATE_EPOCH`. Source archives can supply that value explicitly.
Without source metadata or an explicit value, the timestamp uses the Unix epoch.

Run the mount-order and metadata regression without KVM:

```sh
sudo python3 image/check-export.py \
  --work-dir .context/ciproxy-mvp/orchestrator-fix-1
```

It needs `chroot`, `ldd`, Bash, Python, tar, zstd, e2fsprogs, `acl`, and
`setcap` from the Linux capability tools. It keeps all fixtures as evidence.
It runs provisioning inside a fixture chroot with stub provisioning tools.
It runs the real export script with stub libguestfs transport boundaries.
Real archive, ext4, and package tools check the release tree, the ext4 disk,
and their metadata. Both must keep the owners, modes, file capabilities,
ACLs, and hard links of the source.
The release must use the software report of the pinned recipe, not a stale
report in the unpinned recipe folder.
A package that differs from its APT pin must appear in the measurement as
`apt_pin_differences`, also when the release tree has no APT preference
file. The export must still finish.
The provision step must remove the build-only files of ciproxy before the
export, and the final tree must have none of them. The upstream files of
`/imagegeneration` must stay.
The ext4 check verifies the approved 1% root reserve.
It also checks neutral root labels in the release measurement and manifest.
It checks the small root's free margin and both `root_disk_bytes` fields
against the actual file length. The fixture uses real ext4 resizing tools.
It grows a copy of the packaged release disk with plain `resize2fs`.
It neither forces resizing nor repairs the copy first.
This check cannot prove real guestfish inspection, mounts, or a VM boot.

## Guest network

Check the probe's unit-state handling without systemd or KVM:

```sh
python3 image/check-network-probe.py
```

The export check also checks the generated network file in the source,
the finalized tree, and the ext4 disk. DHCP must be off.
The complete kernel configuration must stay in place. The image supplies
DNS and does not duplicate the agent's address or gateway.

Run the live network smoke on a disposable KVM machine before publication:
Finish its jobs and stop its agent before this probe.
Run it before a fresh agent installation and registered-job scenarios.

```sh
cargo build --package ciproxy-agent --features filesystem-regression \
  --example network-smoke-driver
sudo python3 image/smoke-network.py .context/ciproxy-image-build/artifacts \
  --tools-dir /var/lib/ciproxy/tools/v1.17.0 \
  --network-driver target/debug/examples/network-smoke-driver
```

Use the checked Firecracker and jailer directory from host preparation.
The probe needs `ip`, nftables, systemd, e2fsprogs, and root access.
The opt-in driver calls the production agent network and VM configuration
code. It supplies the boot arguments, namespace, and both firewall paths.
It boots a private disk copy with the same jailer and network setup.
Its temporary collector runs before the guest service and any runner.
No registration, GitHub token, or dashboard credential is needed.
The guest must resolve `github.com` and open a verified HTTPS connection.
Failure returns 1. Missing root or KVM returns SKIP with exit 77.
The probe stops its VM, removes its namespace and disk copy, and keeps safe
network evidence under `.context/image-network-smoke/`.
An unknown unit state fails the probe and retains its named VM resources.
`--disk-bytes` can use the grown copy size from the agent sizing helper.

Add `--initialization-failure` to check the image's failure cleanup hook.
After the network checks, an invalid guest port causes an early failure.
The probe requires an observed guest reboot and automatic VMM exit.
It sends no runner registration. The offline export check executes the
installed cleanup command and checks that both artifacts keep that unit.
The [Firecracker shutdown contract][reset] explains the x86 reset path.

Boat keeps its provider kernel, network services, and resolver. The Boat
build installs none of the image's network configuration. The guest network
smoke covers Firecracker only.

[reset]: https://github.com/firecracker-microvm/firecracker/blob/v1.17.0/FAQ.md#how-can-i-gracefully-reboot-the-guest-how-can-i-gracefully-poweroff-the-guest

## Guest and Boat scripts

These checks need no KVM, GitHub, or Boat account.
Each check runs real image code. Fixtures replace the VM, the disk, the
runner, or the provider that the code would reach.
A check with `--work-dir` keeps its fixtures there as evidence.
Use a folder under `.context/`.

### Runner account environment

Build the guest first. The check starts that binary:

```sh
cargo build --package ciproxy-guest
sudo python3 image/check-runner-environment.py \
  --work-dir .context/image-checks/runner-environment
```

It starts the guest in `sandbox` mode in a chroot with the real `runuser` and
`runner-entry.sh`. A fixture listener records its environment.
The check needs Bash, GNU coreutils, `runuser`, PAM, `ldd`, and `chroot`.
It needs root to make a private account database in the chroot.

The listener must get `HOME`, `USER`, `LOGNAME`, and `SHELL` from the
runner account. Its `PATH` must keep the image tools and add the account's
own entries. The values of `/etc/environment` and the account's `.profile`
must arrive. The JIT registration must arrive unchanged.
The assignment socket value must arrive empty in `sandbox` mode.
No value of the root service environment can reach the listener.

### Docker disk attachment

```sh
sudo python3 image/check-docker-attach.py \
  --work-dir .context/image-checks/docker-attach
```

It runs `docker-attach.sh` in a chroot with stub mount and service tools.
The stub refuses a mount move, as a shared mount does.
The script must unmount its preparation mount and mount `/dev/vdb` directly
at `/var/lib/docker`. The check then makes each step fail in turn.
Each failure must leave both mount points unmounted and Docker running on
the root disk, with its data unchanged.
It needs root for `chroot` and its private `/proc` mount.

### Boat build

```sh
python3 image/check-boat-build.py \
  --work-dir .context/image-checks/boat-build
```

It runs the real build executor and the real pinned renderer on fixture
recipes. The [Boat build check](boat-build-checks.md) lists what it covers
and the tools that it needs.

### Boat launch

Build the guest first. The check starts that binary:

```sh
cargo build --package ciproxy-guest
sudo python3 image/check-boat-run.py \
  --work-dir .context/image-checks/boat-run
```

It runs `boat-run.sh` in a fixture root with a `systemctl` spy and the
real guest. When the launch queues `ciproxy-boat-start.service`, the spy
runs the real command of that unit. A restart of `user@1000.service`
fails unless it runs in that unit, as a restart of Boat's user manager
would end a command in its scope.
With a registration, the launch must reload systemd and queue the start
unit. The start unit must reload systemd, stop Docker and its socket,
restart containerd, and restart each other listed service. It must then
start Docker and its socket, and then the guest unit.
The delivered file must arrive at `/run/ciproxy/registration.json` with
mode 0600 and root as its owner. The source file must be gone, and the
output must not show its contents.
Without a registration, the launch must take the same steps. The check
runs the guest unit's command. The guest must end with exit code 1 and
its missing-registration error.
The fixture root holds the real Python, the real `measure.py`, and a `du`
that records each call and fails.
With a registration, the start must not call `du`. A fixed `statvfs`
sample has different block sizes and different free and available block
counts. The start must measure `/` and print exactly the size as
`backing_filesystem_bytes`, and the available bytes as
`backing_filesystem_available_bytes` and `free_job_bytes`.
When `statvfs` fails in the start, the start must print
`Warning: Boat job disk measurement failed.`, take each refresh step, and
still start the guest unit.
Outside job mode, `measure.py` must still call `du`, fail when `du` fails,
and print nothing.
The check needs root, `chroot`, and `ldd`, so that the fixture root stays
private.

### Boat runner start

```sh
sudo python3 image/check-boat-runner.py
```

It runs the real NVM guard on a fixture profile. It runs the real Boat
runner files in a private root with a fixture listener. The
[Boat runner check](boat-runner-checks.md) lists what it covers and the
tools that it needs.

### Boat metadata comparison

```sh
sudo python3 image/check-boat-metadata.py \
  --work-dir .context/image-checks/boat-metadata
```

It runs `boat.py metadata` locally on a fixture root with a real file
capability and a real ACL. The comparison must match the earlier baseline.
A changed mode must show as a difference. The check needs root for the real
capability and `setfacl` from the ACL tools.

### Boat delete confirmation

```sh
python3 image/check-boat-delete.py
```

It runs `boat.py delete` for two sandbox IDs with a stub HTTP opener.
Each `DELETE` request must name its sandbox and send the same ID in the
`X-Ascii-Confirm-Delete` header. The check needs no Boat key or network.

## Retained userspace export

To export a retained, complete release tree without software provisioning:

```sh
bash image/export.sh "$PWD/.context/ciproxy-image-build" from-rootfs
```

This uses `WORK_DIR/rootfs` and the retained kernel and software reports.
The generated software report must be in the pinned recipe folder that
[paths.env](paths.env) names.
The export removes the Boat files that older exports put in the tree:
`boat-run.sh`, `file-capabilities.py`, and the capability map.
It rebuilds the root disk and the Firecracker package from those inputs.

## Release growth proof

The measured root disk is 60,263,759,872 bytes. The base margin is 64 MiB.
The producer measures empty ext4 metadata and keeps the 1% root reserve.
The agent adds its filesystem allowance for ext4 metadata and reserves.

The proof calls the real `ciproxy_agent::root_sizing::RootSizing` from
commit `d6d18e6dca141fc822232830109ee34c29a3ce95` of the agent sizing branch.
That branch is not merged into the image branch.
Each row uses a fresh sparse copy and the helper's returned length, `R`.
Plain `resize2fs` 1.46.5 grows the copy without `-f` or a prior repair.
A read-only Firecracker probe then runs `df` as runner UID 1001.

| Requested free bytes | Helper `R`, bytes | Measured free bytes |
| ---: | ---: | ---: |
| 1,000,000,000 | 61,311,987,712 | 1,557,065,728 |
| 20,000,000,000 | 80,899,612,672 | 20,641,435,648 |
| 60,000,000,000 | 122,136,727,552 | 60,811,378,688 |
| 500,000,000,000 | 575,744,974,848 | 502,754,795,520 |

The default job leaves 1,600,387,328 bytes of the 82,500,000,000-byte job
area on the 250 GB minimum machine. This exceeds the required 1 GB spare.
The source disk stays clean and unchanged during these copy-based proofs.
These probes measure free space before jobs write data.
Registered-job and full service checks remain separate release work.
