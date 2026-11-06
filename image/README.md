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
Its root arguments include `root=/dev/vda rw net.ifnames=0`.
Each label states free job disk. `RootSizing` adds it and a filesystem
allowance for ext4 metadata and reserves to the selected image's
`root_disk_bytes`. The agent grows each job's reflink copy and ext4 before
boot. Docker uses the root disk unless a sticky Docker disk for the job's
checked scope attaches.
Set its network and DNS through the agent's VM network setup.
The image enables DHCP on `eth0`.
The guest listens on vsock port 5252 by default in its systemd unit.
Change `/etc/ciproxy/guest.env` when the agent uses another port.
The agent must send one JSON object per line over that connection.

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

Use Python 3.9 or later on the host and set `BOAT_API_KEY` there.
The scripts read the key from the environment.
They never pass it into a sandbox.
The [Boat API](https://docs.boat.dev/api/v1) defines these operations.
There is no raw disk-image import operation.

Create a clean template with enough disk for the archive and extraction.
Select a size that the Boat account can start.
The example uses `large` for preparation only.
Wait until the sandbox is ready before the upload.

```sh
python3 image/boat.py create --size large --template
python3 image/boat.py status TEMPLATE_SANDBOX_ID
python3 image/boat.py prepare TEMPLATE_SANDBOX_ID \
  .context/ciproxy-image-build/artifacts/rootfs.tar.zst
python3 image/boat.py status TEMPLATE_SANDBOX_ID --process PREPARE_PROCESS_ID
python3 image/boat.py metadata TEMPLATE_SANDBOX_ID \
  > .context/rootfs-metadata-before.json
python3 image/boat.py snapshot TEMPLATE_SANDBOX_ID IMAGE_RELEASE_NAME \
  --share-file .context/ciproxy-image-share.json
python3 image/boat.py delete TEMPLATE_SANDBOX_ID
```

Require a successful preparation exit before taking the snapshot.
The preparation removes the archive after extraction.
It retains the full userspace under `/opt/ciproxy/rootfs`.
Boat saves that directory as changed files.
The snapshot contains no runner registration, callback token, or App key.
Use a new release name so an existing share keeps the same image.
`snapshot` waits for the saved snapshot and writes its share code to a
new private file. Copy that code into the control plane's image settings.
`metadata` runs [metadata-smoke.py](metadata-smoke.py) on the real
`usr/bin/ping` file before the snapshot. It reads its owner, mode,
capability, and ACL attributes. Keep this evidence on the host.
Confirm that `security_capability` has a value before saving the snapshot.

## Boat smoke

Run the following sequence with `large`, as the approved full image requires.
Use `--trial` with `create` on a free trial to request its two-hour limit.
The default lifetime is six hours.
Keep the returned sandbox ID and process ID in the release evidence.

```sh
python3 image/boat.py create --size large \
  --share-file .context/ciproxy-image-share.json
python3 image/boat.py status SMOKE_SANDBOX_ID
python3 image/boat.py metadata SMOKE_SANDBOX_ID \
  --baseline .context/rootfs-metadata-before.json
python3 image/boat.py run SMOKE_SANDBOX_ID .context/sandbox-registration.json
python3 image/boat.py status SMOKE_SANDBOX_ID --process RUNNER_PROCESS_ID
python3 image/boat.py delete SMOKE_SANDBOX_ID
```

The metadata comparison runs after shared-snapshot restore and before a job.
It prints `matches_baseline` and each difference as smoke evidence.
Confirm that `matches_baseline` is `true` after each restore.
It adds no build gate. Investigate a difference before using the image.
Use `--file RELATIVE_PATH` in both metadata commands to check another file.
For an ACL restore proof, choose a file with a populated ACL attribute.
A `null` ACL records its absence and does not prove ACL restoration.
The metadata script needs Python with Linux extended-attribute support.

For each sandbox, mint a new JIT registration and a token for that sandbox.
The existing control-plane callback must validate that token and delete
that sandbox. Supply its actual HTTPS URL in the private file.
That endpoint belongs to the overflow backend integration.
These scripts do not choose its route or create a hosted endpoint.
Keep the real callback URL and the issued sandbox token in separate files.
Run the App helper on the host with Python and OpenSSL:

```sh
python3 image/github.py --app-id APP_ID --app-key /PRIVATE/APP.pem \
  --installation-id INSTALLATION_ID registration --organization ORGANIZATION \
  --runner-group-id GROUP_ID --runner-id UNIQUE_RUNNER_ID \
  --sandbox-id SMOKE_SANDBOX_ID --callback-url-file .context/callback-url \
  --sandbox-token-file .context/sandbox-token \
  --output .context/sandbox-registration.json
```

The helper signs an App JWT and obtains an installation token in memory.
It writes only the JIT registration and sandbox callback credentials to the
new mode 0600 output file.
Its stdout gives the GitHub runner ID for result correlation.
Use this mode 0600 file format:

```json
{
  "configure": {
    "type": "configure_runner",
    "protocol_version": 1,
    "runner_id": "ONE_RUNNER_ID",
    "organization_id": "GITHUB_ORGANIZATION_ID",
    "registration": "ENCODED_JIT_REGISTRATION",
    "cache": null
  },
  "exit_report_url": "https://callback.example/ACTUAL_SANDBOX_ROUTE",
  "exit_report_token": "TOKEN_FOR_THIS_SANDBOX_ONLY"
}
```

The upload uses a private temporary directory.
No credential enters Boat's command text or a saved snapshot.
The wrapper removes the uploaded file and uses a private runtime mount.
It stops Boat's outer Docker daemon before it starts the image's daemon.
The daemon and runner use the same full userspace as Firecracker.
The wrapper uses private mount and PID namespaces.
The sandbox's network namespace and provider kernel remain in use.
Verify Docker networking and `docker run` in the live smoke job.

Copy [smoke-workflow.yml](smoke-workflow.yml) to a private smoke repository.
Queue the job with an authorized user account or use an already queued job.
The ciproxy App has Actions read permission and cannot dispatch workflows.
Give the runner the workflow's `ciproxy-image-smoke` label.
Keep the App credential on the host.
Record GitHub's actual job result separately from the listener's exit code.
Read the known workflow job with the App's existing Actions read permission:

```sh
python3 image/github.py --app-id APP_ID --app-key /PRIVATE/APP.pem \
  --installation-id INSTALLATION_ID result --repository OWNER/REPOSITORY \
  --job-id GITHUB_JOB_ID
```

Require `completed`, `success`, and the expected runner ID in that result.
The wrapper prints free bytes before the runner starts.
A successful callback prints `runner exit report accepted`.
Verify the final Boat state as well. A successful job alone does not prove
that the callback deleted its sandbox.
If the control plane deletes it before status can read the logs, keep the
callback server's receipt and the provider deletion result as evidence.
Delete a remaining smoke sandbox with the explicit delete command.

Boat documents 51 GB for files on `small` and `default`.
Those sizes cannot hold the full userspace.
Keep the full userspace on `large` and measure its actual free job disk.

## Sources

The [source list](sources.md) links the pinned recipes and provider docs.
