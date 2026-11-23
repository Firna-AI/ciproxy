# The full ciproxy job image

This Apache-2.0 image definition keeps the full Ubuntu 24.04 x64 software
recipe of `actions/runner-images`.
Each release builds it twice, with the same recipe and version pins.
The machine build makes the Firecracker disk, which boots the bundled Linux
kernel.
The [Boat build](boat.md) installs the same software on Boat's own Ubuntu
system and saves it as the Boat snapshot.
The stock runner is the same in both.
Boat keeps its provider kernel, base packages, and provider services.
Only the Boat build guards one line of Boat's NVM profile and wraps the
runner entry. The
[Boat build steps](boat-build-steps.md#the-nvm-profile-of-boats-base) say
why.
[Agent images](../docs/protocol/agent-images.md) gives the requirements and
each difference that stays.
The [guest guide](../crates/ciproxy-guest/README.md) explains supervision.

## Inputs and build

[pins.env](pins.env) fixes the runner-images commit, Ubuntu base disk,
software inventory, stock runner, Linux source, and Firecracker kernel
configuration by version and SHA-256.
The runner-images pin is `ubuntu24/20261101.360` at
`add4380b0d6685806e9192c33c37359ee0aa4023`.
Its inventory is the reference for the full software set.
The recipe selects some software at its latest version.
[software-pins.json](software-pins.json) pins each of these selections,
such as tool versions, release assets, and container image digests.
[software-apt-pins.json](software-apt-pins.json) holds every APT
preference of both builds in its `packages` map.
Each pin uses the version of the reference inventory when the inventory
names one. When a repository no longer offers that version, the pin uses a
version that the repository offers. `selection_notes` explains each pin
that differs from the reference. The file also records the repository
`sources` and the date of its last check, `checked_on`.
The other pins record a concrete version from their source. The `sources`
notes of `software-pins.json` explain a pin that its source cannot match
exactly. An example is the Chromium pin, which is the newest snapshot at or
below the pinned Chrome revision.
Some pins keep the selector form that the recipe expects. For example, the
toolset keeps the upstream Node.js default, and the install command names
the exact pinned Node.js version.
Both builds apply these pins to a copy of the recipe, the pinned recipe.
[paths.env](paths.env) names its folder in `WORK_DIR`.
In the pinned recipe, no package request overrides its APT pin.
A GitHub release lookup uses the pin of the repository when one exists.
An exact version without a pin selects the one release whose tag names
that version. `latest` or a pattern without a pin keeps the upstream
lookup. The lookup reads the prepared release answers first, and then the
answers that the build saved earlier. The machine build saves each answer
that it fetches in `/tmp/ciproxy-github-cache`, outside `/imagegeneration`.
Packages that no pin covers still resolve from their repositories at build
time. Both builds compare each installed package with the `packages` map,
and report each package whose version differs from its pin. No difference
fails a build. The machine build adds this report to `measurement.json` as
`apt_pin_differences`, and the manifest includes that measurement.
The report gives `available`, `difference_count`, `differences`, and
`differences_fail_build`, which is always `false`. Each difference gives
`package`, `installed`, and `pin`. A vertical bar joins several matching
pins. When the build cannot read the pin map or the package status,
`available` is `false`, and `reason` says why.
Review the generated software inventory and these differences before
publishing.

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
They also turn on zram, as [Compressed swap](#compressed-swap) says.
The build user needs KVM access and `sudo` for extraction and ext4 creation.
Run from the repository root:

```sh
bash image/build.sh "$PWD/.context/ciproxy-image-build"
```

This downloads and checks the inputs, builds the kernel and guest, and
runs the full provisioners of the pinned recipe in QEMU.
QEMU supplies the image packaging.
It retains the upstream software tests and software report.
The QEMU bootstrap prepares the generic Ubuntu package sources for the
upstream mirror-list setup and its existing check.
`prepare-template.py` replaces the Azure source and deprovision command.
It adds the ciproxy guest, runner, kernel, and service after provisioning.
No Azure account is needed.
The ciproxy provision step at the end of the machine build runs
[boat-build-clean-image.sh](boat-build-clean-image.sh). It removes the
build-only files of ciproxy, such as the answer cache in
`/tmp/ciproxy-github-cache` and the pin file in `/imagegeneration/helpers`.
The upstream image metadata, helpers, tests, software reports, and
`SoftwareReport` folder stay.
`rootfs-archive.sh` shares the export and extraction metadata policy.
It saves numeric owners, extended attributes, and POSIX ACLs with guestfish.
The extraction restores every saved attribute namespace.
The partitioned source keeps its mount configuration through inspection and
export. `source-rootfs.tar.zst` stays in `WORK_DIR` as an intermediate.
The export extracts one release tree and sets `/dev/vda` in its `etc/fstab`.
The Firecracker ext4 disk comes from that finalized tree.

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

`WORK_DIR/artifacts` holds `rootfs.ext4`, `vmlinux`, the reference
software inventory, the generated software report, the release manifest,
and `SHA256SUMS`.
`firecracker.tar.zst` bundles the sparse ext4 disk, kernel, inner manifest,
inventories, measurement, and `firecracker-content.sha256`.
The agent's single `ImageDownload` URL must point to that bundle.
`downloads.json` gives its actual SHA-256 and byte size.
After extraction, check `firecracker-content.sha256` before boot.
The inner manifest excludes the bundle's own hash to avoid a hash cycle.
Boat uses no release download. The [Boat build](boat.md) installs the
same pinned image definition on Boat's own system.
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
checked scope attaches. `docker-attach.sh` prepares that disk, then mounts
it directly at `/var/lib/docker`. After a failed step, Docker runs on the
root disk again.
The agent supplies the guest address and gateway in its kernel arguments.
The image disables DHCP on `eth0` and keeps the complete kernel network
configuration, including its boot routes. The image supplies DNS through
its network configuration. No guest address or gateway is set in the image.
The [network smoke](checks.md#guest-network) checks public DNS and HTTPS
before any runner starts.
It keeps the VM resources when the host cannot confirm that the VM stopped.
The guest listens on vsock port 5252 by default in its systemd unit.
Change `/etc/ciproxy/guest.env` when the agent uses another port.
The agent must send one JSON object per line over that connection.
The image service requests a guest reboot after exit, including an early
initialization failure. With the agent's `reboot=k` argument, this reset
ends x86 Firecracker. The Boat snapshot has its own guest unit with no
reboot, as [Boat launch](boat.md#boat-launch) says.

The [checks and growth proofs](checks.md) cover the bootstrap and artifacts.
Offline checks need no KVM. They grow a release copy with plain `resize2fs`,
without forcing it.

## Compressed swap

Each Linux job starts with a compressed swap in memory, on zram.
[Agent images](../docs/protocol/agent-images.md#one-definition-two-builds)
gives the requirement and the size of the swap.
The pinned Firecracker kernel configuration has zram off.
`build-kernel.sh` turns on `CONFIG_ZRAM`, `CONFIG_ZRAM_BACKEND_ZSTD`, and
`CONFIG_ZRAM_DEF_COMP_ZSTD` with the kernel's `scripts/config` before
`olddefconfig`.
The options are built in, because the image has no modules for the job VM
kernel.
`olddefconfig` drops an option without an error when a dependency is
missing. So the kernel build stops with an error when one of the three
options is not `y` after `olddefconfig`.

In both builds, `provision.sh` installs `ciproxy-swap.sh` as
`/usr/local/libexec/ciproxy-swap` and installs `ciproxy-swap.service`.
The machine image enables the unit. It runs the script at boot, before
`ciproxy-guest.service`.
A Boat restore keeps the running system. `boat-start.sh` starts the unit
after Docker and before the guest. It warns and goes on if the unit fails.
The script changes nothing when a zram swap is already active.
When zram is a module, the script loads it.
It sizes a zram device at `MemTotal` of `/proc/meminfo`, but at most
8 GiB, with the zstd compressor.
Without zstd, it warns and keeps the default compressor of the kernel.
It turns the swap on with priority 100, so the kernel fills it before a
disk swap that Boat adds.
It sets `vm.page-cluster` to 0, because zram has no seek cost.
It does not change `vm.swappiness`.
When zram is missing, or another step fails, the script prints a warning
and exits 0. It leaves no partial swap behind, so the job runs without
swap. Only a failed `vm.page-cluster` write keeps the swap, because that
setting only tunes it.
The [swap check](checks.md#compressed-swap) covers these cases.

## Publish and keep the runner current

Before you build a Linux release with changed image contents, set a new
`IMAGE_ID` in [pins.env](pins.env). Machines and the control plane compare
only the image ID to decide whether a machine needs an update.

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
Repeat the machine build, publication, the Boat build and snapshot, and
both smoke jobs when the pins change.

## Boat shared snapshot

Follow the [Boat build guide](boat.md#boat-shared-snapshot) after the
machine build. It installs the same image definition on Boat's own system,
with the same pins and the same built ciproxy parts. It then saves the Boat
snapshot.

## Boat smoke

The [Boat smoke guide](boat-smoke.md) covers metadata, a restore without
a registration, a real GitHub job, and callback cleanup. The restore part
first runs [boat-locale-smoke.py](boat-locale-smoke.py). That check
requires the `UTF-8` codeset and an empty stderr for the first command in
the restored sandbox. The restore part also runs the runner entry with no
registration, which must reach its registration check. Boat keeps its
provider kernel, network, and resolver.

## Sources

The [source list](sources.md) links the pinned recipes and provider docs.
