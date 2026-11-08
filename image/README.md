# The full ciproxy job image

This Apache-2.0 image definition keeps the full Ubuntu 24.04 x64 software
recipe of `actions/runner-images`.
One built userspace supplies Firecracker disks and Boat snapshots.
The stock runner is the same in both.
The bundled Linux kernel boots Firecracker.
Boat keeps its provider kernel and services.
The [guest guide](../crates/ciproxy-guest/README.md) explains supervision.

## Inputs and build

[pins.env](pins.env) fixes the runner-images commit, Ubuntu base disk,
software inventory, stock runner, Linux source, and Firecracker kernel
configuration by version and SHA-256.
The runner-images pin is `ubuntu24/20261101.360` at
`add4380b0d6685806e9192c33c37359ee0aa4023`.
Its inventory is the reference for the full software set.
Some upstream provisioners resolve remote package repositories at build
time. Review the generated software inventory before publishing.

Use a disposable x64 Linux build machine with KVM and enough disk for the
base disk, build disk, Linux source, archive, extraction, and final disk.
Each copy can exist at the same time.
The build VM uses the CPU of the build machine.
The build machine needs the CPU features of the software set, such as
SSSE3 and AVX.
The QEMU build root uses Virtio-SCSI. It appears as a real SCSI disk for
the upstream read-ahead check. The exported Firecracker root stays ext4
on `/dev/vda`.
The installed size must be measured after the full build.
Upstream's 75 GB virtual disk allocation does not measure installed bytes.

Install Packer, QEMU, `xorriso`, `libguestfs-tools`, `zstd`, `e2fsprogs`,
`curl`, `jq`, Python 3.9 or later, and the repository's Rust toolchain.
The libguestfs tools must include the `guestfish` command.
Kernel builds also need `make`, GCC, `flex`, `bison`, `bc`, `libelf-dev`,
and `libssl-dev`.
Kernel builds use [neutral metadata](checks.md#kernel-build-metadata)
and a source-derived timestamp.
The build user needs KVM access and `sudo` for extraction and ext4 creation.
Run from the repository root:

```sh
bash image/build.sh "$PWD/.context/ciproxy-image-build"
```

This downloads and checks the inputs, builds the kernel and guest, and
runs the full upstream provisioners in QEMU.
QEMU supplies the image packaging.
It retains the upstream software tests and software report.
The QEMU bootstrap prepares the generic Ubuntu package sources for the
upstream mirror-list setup and its existing check.
`prepare-template.py` replaces the Azure source and deprovision command.
It adds the ciproxy guest, runner, kernel, and service after provisioning.
No Azure account is needed.
`rootfs-archive.sh` shares the export and extraction metadata policy.
It saves numeric owners, extended attributes, and POSIX ACLs with guestfish.
Both extraction paths use it to restore every saved attribute namespace.
The partitioned source keeps its mount configuration through inspection and
export. `source-rootfs.tar.zst` stays in `WORK_DIR` as an intermediate.
The export extracts one release tree and sets `/dev/vda` in its `etc/fstab`.
Both the Boat archive and Firecracker ext4 disk come from that finalized tree.

To prepare inputs and the recipe without starting a VM:

```sh
bash image/build.sh "$PWD/.context/ciproxy-image-build" prepare-only
```

The output root contains the full userspace, ext4 metadata, a 1% root
reserve, and a small free margin. It contains no label's job allowance.
`create-root-disk.py` measures empty ext4 layouts before copying the tree.
It includes the allocated userspace, that metadata, and the root reserve.
It keeps a 64 MiB base margin and rounds growth to complete block groups.
That margin leaves 48 MiB beyond the kernel's 16 MiB reserve pool.
The agent supplies each label's growth headroom through `RootSizing`.
The export never mounts the released disk writable after creation.
After any writable mount, run `e2fsck -fp` before checksums and publication.

`ROOTFS_BYTES` sets an explicit logical allocation for a measurement run.
That override bypasses automatic sizing. It does not change a ciproxy label.

The measured full userspace is 55.22 GB apparent and 58.29 GB allocated.
The measured root disk is 60,263,759,872 bytes.

The agent's default image budget is 140 GB for two images and one
download during an update. This budget is separate from job disk space.
It counts allocated blocks. The release measurement and download manifest
record the current artifact sizes.
Keep the full software set. Do not remove software to meet a disk budget.

The [retained export guide](checks.md#retained-userspace-export) exports a
complete release tree without software provisioning.

## Artifacts and measurements

`WORK_DIR/artifacts` holds `rootfs.ext4`, `rootfs.tar.zst`, `vmlinux`, the
reference software inventory, the generated software report, the release
manifest, and `SHA256SUMS`.
`firecracker.tar.zst` bundles the sparse ext4 disk, kernel, inner manifest,
inventories, measurement, and `firecracker-content.sha256`.
The agent's single `ImageDownload` URL must point to that bundle.
`downloads.json` gives its actual SHA-256 and byte size.
After extraction, check `firecracker-content.sha256` before boot.
The inner manifest excludes the bundle's own hash to avoid a hash cycle.
The Boat download remains `rootfs.tar.zst`.
`package.sh ARTIFACT_DIR` can repeat the bundle and checksum step.
Each artifact records its logical and allocated bytes.
The measurement and release manifest expose `root_disk_bytes` at the top
level. This is the actual logical length of `rootfs.ext4` in SI bytes.
`write-manifest.py` copies this measured value into the manifest.
The installed userspace records its apparent and allocated bytes.
The measurement uses the neutral `rootfs` label instead of a machine path.
The ext4 record uses the image's actual block geometry for offline free
bytes. The extraction's backing filesystem has separate fields.
Record actual boot and job free-disk measurements from the running VM or
Boat filesystem with the release evidence.

Firecracker uses the unpartitioned ext4 disk and the supplied `vmlinux`.
Its root arguments include `root=/dev/vda rw`.
Each label states free job disk. `RootSizing` adds it and a filesystem
allowance for ext4 metadata and reserves to the selected image's
`root_disk_bytes`. The agent grows each job's reflink copy and ext4 before
boot. Docker uses the root disk unless a sticky Docker disk for the job's
checked scope attaches.
The agent supplies the guest address and gateway in its kernel arguments.
The image disables DHCP on `eth0` and keeps the complete kernel network
configuration, including its boot routes. The image supplies DNS through
its network configuration. No guest address or gateway is set in the image.
The [network smoke](checks.md#guest-network) checks public DNS and HTTPS
before any runner starts.
The guest listens on vsock port 5252 by default in its systemd unit.
Change `/etc/ciproxy/guest.env` when the agent uses another port.
The agent must send one JSON object per line over that connection.
The image service requests a guest reboot after exit, including an early
initialization failure. With the agent's `reboot=k` argument, this reset
ends x86 Firecracker. Boat does not start this systemd service.

The [checks and growth proofs](checks.md) cover the bootstrap and artifacts.
Offline checks need no KVM. They grow a release copy with plain `resize2fs`,
without forcing it.

## Publish and keep the runner current

Use an authenticated Google Cloud CLI on the release machine.
The destination is the image bucket that the deployment provides.

```sh
bash image/publish.sh .context/ciproxy-image-build/artifacts \
  gs://IMAGE_BUCKET/RELEASE_PREFIX
bash image/check-runner.sh
```

The publish script checks every SHA-256 before upload.
Serve those checksums with the artifacts through the download site.
Check for a new runner each week and rebuild for each release.
Ship a required security update at once.
GitHub can refuse an old runner after a newer release, or at once for a
critical security update.
The release check prints version data. A live smoke job proves acceptance.
Repeat the full image build, publication, Boat snapshot, and both smoke
jobs when the pins change.

## Boat shared snapshot

Follow the [Boat preparation guide](boat.md#boat-shared-snapshot) with the
same checked userspace archive as Firecracker.

## Boat smoke

The [Boat smoke guide](boat.md#boat-smoke) covers metadata, a real GitHub
job, and callback cleanup. Boat keeps its provider network and resolver.

## Sources

The [source list](sources.md) links the pinned recipes and provider docs.
