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
  --worker .context/ciproxy-image-build/worker-overlay \
  --runner .context/ciproxy-image-build/runner.tar.gz \
  --reference .context/ciproxy-image-build/artifacts/software-report.json \
  --output .context/ciproxy-boat-build.tar.gz
```

`bundle` checks the upstream and runner archives against
[pins.env](pins.env). It renders the pinned recipe, as the machine build
does. It then changes a few upstream scripts and tests for Boat, and keeps
each change with its reason. It also fetches the public GitHub release
answers that the pins name, with no credential. CMake has no `github` pin,
so `bundle` also fetches the release pages up to the release whose tag
names the exact CMake version of the toolset. `--metadata-cache DIRECTORY`
seeds these bundle answers with saved answers.
The bundle keeps the answers outside the pinned recipe. In the sandbox,
they are in `/opt/ciproxy-build/github-answers`, and the build reads them
first. An answer that the build must fetch goes to
`/opt/ciproxy-build/github-cache`. So a change of release data, such as a
download count or a new release, changes no step identity.
The bundle holds the image definition, the pinned recipe, the answers, the
build plan, the reference inventory, and the ciproxy parts.
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
it. It reuses a passed provisioner only when that provisioner and every
earlier one are unchanged. So a changed or new earlier provisioner makes
each later provisioner run again. A later change never makes an earlier
provisioner run again.
A step that moves a copied folder, such as the merge of `docs-gen` into
`SoftwareReport`, depends on every source of the earlier copy steps that
write that folder. So a copy step that runs again is always followed by its
move. The move merges the folder into its target and keeps the files there.
No copied source folder stays in `/imagegeneration`.
Use `resume` when the cause is gone, such as a network failure.
After a fix to the image definition, write a new bundle and use `refresh`.
`refresh` checks the new bundle, extracts it over the build folder, and
resumes. It replaces the stored bundle answers with those of the new
bundle, so the current bundle answers win over the answers of an earlier
run. It refuses to start while the build still runs.
Keep the build report in the sandbox, because a resume reads it.

A provisioner that runs again finds the files of its earlier run. The PyPy
installer reuses a complete version and replaces its own incomplete version
folder. Other installers can fail on their own earlier files. A replay
failure of that kind needs a new build from a clean Boat source.
The first step keeps its first records and does not repeat its cleanup.
So a change to the state before the recipe steps, such as a new cleanup
rule, also needs a new build from a clean Boat source.

The [Boat build steps](boat-build-steps.md) say what the build does to
Boat's base, which recipe steps it skips or changes, and which reports it
writes.

Require a passed build report before `finalize`.
`finalize` copies the report, its files, and the provisioner logs to the
`--output` folder on the host. When a copy fails, it stops and keeps the
build files. It then runs
[boat-build-clean-image.sh](boat-build-clean-image.sh), which removes the
build-only files of ciproxy: its pin file in `/imagegeneration/helpers`, a
release cache that older builds left there, the machine answer cache, the
build apt settings, and the provider Docker group file. Then it removes
`/opt/ciproxy-build`, with both answer stores, and the temporary ciproxy
files. The upstream content of `/imagegeneration` and
`/opt/post-generation` stays, as in the machine image.
The cleanup script needs no build folder, and a second run changes nothing.
So it can also clean a sandbox from an older snapshot. Upload it to
`/tmp/boat-build-clean-image.sh`, run
`sudo bash /tmp/boat-build-clean-image.sh`, and remove the script. Then
save the new snapshot.
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
5. It prints the size of the root filesystem as `backing_filesystem_bytes`
   and the bytes that a job can use as `free_job_bytes` to its journal. It
   reads them with `statvfs` and scans no files. If this measurement fails,
   it writes `Warning: Boat job disk measurement failed.` and goes on.
6. It starts `ciproxy-boat-guest.service`.

A failure of the service refresh stops the start before the guest starts.

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
