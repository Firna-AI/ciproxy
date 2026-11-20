# Boat image build and launch

The Boat snapshot is the Boat build of the job image.
It installs the same image definition and version pins as the
[machine build](README.md#inputs-and-build) on Boat's own Ubuntu system.
[Agent images](../docs/protocol/agent-images.md#the-boat-snapshot) gives
the requirements and each difference from the machine image.
The [Boat smoke guide](boat-smoke.md) checks a saved snapshot.

## Boat shared snapshot

Use Python 3.9 or later on the host and set `BOAT_API_KEY` there.
The scripts read the key from the environment.
They never pass it into a sandbox.
The [Boat API](https://docs.boat.dev/api/v1) defines these operations.
There is no raw disk-image import operation.
So the build installs the software on the system of a Boat sandbox.

### Build bundle

Run the machine build first. The Boat build uses its checked upstream
archive, its runner archive, its built ciproxy parts, and its software
report as the reference inventory. Write the bundle on the host:

```sh
python3 image/boat-build.py bundle \
  --upstream-archive .context/ciproxy-image-build/runner-images.tar.gz \
  --guest target/release/ciproxy-guest \
  --cache target/release/ciproxy-cache-sidecar \
  --worker .context/ciproxy-image-build/patched-worker/Runner.Worker.dll \
  --runner .context/ciproxy-image-build/runner.tar.gz \
  --reference .context/ciproxy-image-build/artifacts/software-report.json \
  --output .context/ciproxy-boat-build.tar.gz
```

`bundle` checks the upstream and runner archives against
[pins.env](pins.env). It renders the pinned recipe, as the machine build
does. It then changes a few upstream scripts and tests for Boat, and keeps
each change with its reason. It also fetches the public GitHub release
answers that the pins name, with no credential. The build reads these
answers before it asks GitHub. `--metadata-cache DIRECTORY` adds saved
answers first. The bundle holds the image definition, the pinned recipe,
the build plan, the reference inventory, and the ciproxy parts.
It holds no credential.

### Build and save the snapshot

Create a clean template with enough disk for the bundle and the full
software set. Select a size that the Boat account can start.
The example uses `large` for the build only.
Wait until the sandbox is ready before the upload.

```sh
python3 image/boat.py create --size large --template
python3 image/boat.py status TEMPLATE_SANDBOX_ID
python3 image/boat.py prepare TEMPLATE_SANDBOX_ID \
  .context/ciproxy-boat-build.tar.gz
python3 image/boat.py status TEMPLATE_SANDBOX_ID --process PREPARE_PROCESS_ID
python3 image/boat-build.py report TEMPLATE_SANDBOX_ID
python3 image/boat-build.py finalize TEMPLATE_SANDBOX_ID \
  --output .context/ciproxy-boat-build-report
python3 image/boat.py metadata TEMPLATE_SANDBOX_ID \
  > .context/rootfs-metadata-before.json
python3 image/boat.py snapshot TEMPLATE_SANDBOX_ID IMAGE_RELEASE_NAME \
  --share-file .context/ciproxy-image-share.json
python3 image/boat.py delete TEMPLATE_SANDBOX_ID
```

`prepare` uploads the bundle and [boat-prepare.sh](boat-prepare.sh).
The script checks the SHA-256 of the bundle and extracts it to
`/opt/ciproxy-build`. It then starts the build as the
`ciproxy-image-build` systemd unit and returns.
The build continues after the Boat command ends.
`boat-build.py prepare` does the same as `boat.py prepare`.
`report` prints the build report. Repeat it until the build status is
`passed` or `failed`. `report --output DIRECTORY --logs` also copies the
report files and the provisioner logs to the host.

After a failure, read the log of the failed provisioner. Then resume the
build, or refresh it with a new bundle:

```sh
python3 image/boat-build.py report TEMPLATE_SANDBOX_ID --log PROVISIONER_ID
python3 image/boat-build.py resume TEMPLATE_SANDBOX_ID
python3 image/boat-build.py refresh TEMPLATE_SANDBOX_ID \
  .context/ciproxy-boat-build.tar.gz
```

`resume` runs the failed provisioner again and then the provisioners after
it. A provisioner that passed runs again only when its inputs changed.
A step that moves a copied folder, such as the merge of `docs-gen` into
`SoftwareReport`, depends on every source of the earlier copy steps that
write that folder. So a copy step that runs again is always followed by its
move. The move merges the folder into its target and keeps the files there.
No copied source folder stays in `/imagegeneration`.
Use `resume` when the cause is gone, such as a network failure.
After a fix to the image definition, write a new bundle and use `refresh`.
`refresh` checks the new bundle, extracts it over the build folder, and
resumes. It refuses to start while the build still runs.
Keep the build report in the sandbox, because a resume reads it.

The build keeps Boat's controller, desktop services, network, and resolver.
Before it changes Boat's system, it records these facts of Boat's base in
the build folder:

- The group ID of Boat's `docker` group.
- The modes of Boat's own tool files.
- The owners of `/usr`, `/etc`, `/usr/local`, and `/usr/local/bin`.
  Boat's base gives the last two to its `user` account.

It sets apt to confirm each install and to permit a downgrade, so a pinned
version can replace a newer version of Boat's base.
After the package index update, it removes Boat's own commands and shell
settings that shadow or block a tool of the pinned definition. So a job
finds the pinned version first, as on a job VM:

- The commands of CMake, Go, Node, the Ruby family, the Rust family,
  Composer, Gradle, and Kotlin in `/usr/local/bin`. It removes only the
  command entry and does not follow a link.
- The Rust `PATH` element and `RUSTUP_HOME` in `/etc/environment`.
- The Rust profile script `/etc/profile.d/rust.sh`, when no package owns
  it.
- Boat's exact Rust block in `/etc/bash.bashrc`, once. The other lines of
  the file stay. The build then checks the file against the md5 that dpkg
  records for it. A mismatch adds a warning to the record, and the build
  continues.

It keeps Boat's provider and agent commands, the tool homes, and the extra
tools that no pinned tool replaces, such as pnpm, Bun, Deno, uv, Poetry,
sbt, and Scala. It also keeps Boat's nvm profile script and the shell
files of root, because they put no Boat tool before a pinned tool.
It records each removed and kept item. A resume keeps the first records.
When the first step runs again, it does not repeat this cleanup.

The build then points the Ubuntu package sources at the mirror of the
machine build. It runs each provisioner of the pinned recipe in the
recipe's order, as root or as Boat's `user` account, as the recipe says.
The temporary files of the installers stay in the build folder.
The upstream software tests run in the provisioners, as in the machine
build.

The build records each provisioner in `/opt/ciproxy-build/report.json`,
with its status, exit code, duration, and log file.
A skipped provisioner keeps its reason in the same record.
The build skips only these provisioners:

- The Azure apt wrapper. Boat uses the normal apt exit codes.
- The Azure deprovision. It would remove Boat's account and its controller
  access.
- The reboot. Boat restores files onto its own running system.
- The downloads of the software reports. `report` and `finalize` copy
  them to the host instead.

The bundle also changes some upstream scripts and tests for Boat. The main
changes are:

- It removes the Azure swap, resource disk, boot loader, and root mount
  setup, because Boat owns them. It skips the tests of that setup.
- It runs the read-ahead test of the disks only on a SCSI or NVMe disk.
- It applies the upstream writable permissions only to new paths, so
  Boat's own files keep their modes.
- The upstream cleanup and the post-generation log script keep `/tmp` and
  the logs of the running Boat system. Their other cleanup steps stay.
- The Docker installer keeps Boat's `docker` group ID. It restarts the
  installed Docker and containerd before their tests.
- The upstream test `<Directory> is owned by root` becomes
  `<Directory> keeps its owner from before the build`. It checks that
  `/usr`, `/etc`, `/usr/local`, and `/usr/local/bin` keep the owners that
  the build recorded. The machine image keeps the upstream root test.

The report lists each change with its reason.

[boat-provision.sh](boat-provision.sh) then installs the ciproxy parts.
It runs [provision.sh](provision.sh) in its Boat mode. That mode installs
the runner, the guest, the cache sidecar, and the patched worker as on a
machine. It adds the `runner` account to the `sudo` and `docker` groups.
It runs the post-generation scripts of the recipe at build time, as the
machine build does. No start runs them again. It installs no kernel, no
network configuration, and no vsock guest unit.
`/opt/ciproxy-image` gets only the files that a launch uses:
`boat-run.sh`, `boat-start.sh`, `measure.py`, and `metadata-smoke.py`.
The directory also keeps the Apache-2.0 [LICENSE](LICENSE).
The script also adds the `sandbox-workload` account that receives each
delivered registration. It installs `ciproxy-boat-start.service` and
`ciproxy-boat-guest.service` disabled, so a restore alone starts no guest.

After the provisioners, the build writes its reports under
`/opt/ciproxy-build/reports`:

- `software-report.json`, the upstream software report of the Boat build.
- `installed-packages.json`, each installed package and its version.
- `installed-inventory.json`, the same packages, and whether the software
  report exists. It also gives the records that the build made of Boat's
  base system. A record that does not exist is `null`:
  - `provider_docker_gid`, the group ID of Boat's `docker` group.
  - `provider_directory_owners`, the owners of the four directories.
  - `provider_developer_environment`, each command and setting that the
    cleanup removed or kept, with its warnings.
- `inventory-differences.json`, each version difference from the
  reference inventory. It says when the software report is missing.
- `changed-services.json`, the running services whose program or unit file
  the build changed. Docker and containerd are always in this list.

The build also writes that service list to
`/etc/ciproxy/boat-restart-services` for each launch.
A version difference does not fail the build. Record the differences with
the release evidence.

Require a passed build report before `finalize`.
`finalize` copies the report, its files, and the provisioner logs to the
`--output` folder on the host. When a copy fails, it stops and keeps the
build files. It then removes `/opt/ciproxy-build`, the build apt settings,
and the temporary ciproxy files. `/imagegeneration` and
`/opt/post-generation` stay, as in the machine image.
Keep the copied reports with the release evidence.
Use a new release name so an existing share keeps the same image.
`snapshot` refuses a template that has runner data or a delivered
registration. It waits for the saved snapshot and writes its share code to
a new private file. Copy that code into the control plane's image settings.
The snapshot contains no runner registration, callback token, or App key.

`metadata` runs [metadata-smoke.py](metadata-smoke.py) on Boat's root
filesystem. The default file is the real `usr/bin/ping`.
It reads its owner, mode, capability, and ACL attributes.
Keep this evidence on the host.
Confirm that `security_capability` has a value before saving the snapshot.

## Boat launch

[boat-run.sh](boat-run.sh) starts the guest of one Boat job.
The overflow launcher runs it as root once for each delivery, with the path
of the delivered registration file. The script copies the registration to
`/run/ciproxy/registration.json`, which only root can read, and removes the
delivered file. Without a path, it removes an old registration file.
It then reloads systemd, queues `ciproxy-boat-start.service`, and returns.

The start unit is a root service of Boat's systemd. It runs
[boat-start.sh](boat-start.sh) once, with no time limit and no retry.
The refresh runs in this separate service, so a restart of a changed
controller or user service cannot end it.
A restore keeps the running processes of Boat's base system. So the script
brings the restored services up to date before the guest starts:

1. It reloads systemd.
2. It stops Docker and its socket, and restarts containerd.
3. It restarts each other running service in
   `/etc/ciproxy/boat-restart-services`, such as `user@1000.service`.
4. It starts Docker and its socket again.
5. It prints the free bytes of the root filesystem to its journal.
6. It starts `ciproxy-boat-guest.service`.

Without a registration, the guest ends with its missing-registration error
and starts no runner.

The guest unit runs the guest in its Boat mode. It has no restart and no
reboot at exit. The guest is a normal service of Boat's systemd.
The launch uses no `chroot`, no private namespace, and no second Docker
daemon. A job uses the system's Docker service, so
`sudo systemctl restart docker` works as on a machine.
Boat keeps its provider kernel, network, resolver, and firewall.
The image's network configuration is not installed, so it cannot change
Boat's addresses, routes, or DNS.
