# Boat build steps

This page says what the [Boat build](boat.md#boat-shared-snapshot) does in
the template sandbox, between `prepare` and `finalize`.

## Records and cleanup of Boat's base

The build keeps Boat's controller, desktop services, network, and resolver.
Before it changes Boat's system, it records these facts of Boat's base.
Finalization removes the records from the sandbox. The build reports keep
the group ID and the owners:

- The group ID of Boat's `docker` group.
- The modes of Boat's own tool files.
- The owners of `/usr`, `/etc`, `/usr/local`, and `/usr/local/bin`.
  Boat's base gives the last two to its `user` account.

It sets apt to confirm each install and to permit a downgrade, so a pinned
version can replace a newer version of Boat's base.
After the package index update, it removes Boat's own commands, shell
settings, and apt packages that shadow or block a tool of the pinned
definition. So a job finds the pinned version first, as on a job VM:

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
- Boat's apt packages for CMake, .NET 8, Maven, and PHP 8.4. The build
  first simulates the purge of each group. When apt would also remove
  another package, the build keeps the whole group and records a warning.
  Otherwise it purges the group, also packages that keep only their
  configuration files.

It keeps Boat's provider and agent commands, the tool homes, and the extra
tools that no pinned tool replaces, such as pnpm, Bun, Deno, uv, Poetry,
sbt, and Scala. It also keeps Boat's nvm profile script and the shell
files of root, because they put no Boat tool before a pinned tool.
It records each removed and kept item. A resume keeps the first records.
When the first step runs again, it does not repeat this cleanup.

## Recipe steps

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
- It gives each folder that the upstream recipe makes writable, such as
  `/opt`, mode 777, as upstream does. Boat's existing entries below that
  folder keep their modes and owners. New entries get mode 777.
- The upstream cleanup and the post-generation log script keep `/tmp` and
  the logs of the running Boat system. Their other cleanup steps stay.
- The Docker installer keeps Boat's `docker` group ID. It restarts the
  installed Docker and containerd before their tests.
- The upstream test `<Directory> is owned by root` becomes
  `<Directory> keeps its owner from before the build`. It checks that
  `/usr`, `/etc`, `/usr/local`, and `/usr/local/bin` keep the owners that
  the build recorded. The machine image keeps the upstream root test.
- The PyPy installer reuses a complete version. It replaces only its own
  incomplete version folder before it installs again.
- Native steps run GNU tar with `--no-overwrite-dir`. So an archive that
  extracts over an existing folder keeps the owner and mode of that folder.
  A new folder gets the metadata of the archive. The machine build is
  unchanged.

The report lists each change with its reason.

## The ciproxy parts

[boat-provision.sh](boat-provision.sh) then installs the ciproxy parts.
It runs [provision.sh](provision.sh) in its Boat mode. That mode installs
the runner, the guest, the cache sidecar, and the patched worker as on a
machine. It adds the `runner` account to the `sudo` and `docker` groups.
It installs [boat-runner-entry.sh](boat-runner-entry.sh) at
`/usr/local/libexec/ciproxy-runner`, the path that the guest runs. It
installs the common [runner-entry.sh](runner-entry.sh) as
`/usr/local/libexec/ciproxy-runner-common`, and the two helpers that
[The runner stderr of a Boat job](#the-runner-stderr-of-a-boat-job)
describes. The machine build keeps `runner-entry.sh` at
`/usr/local/libexec/ciproxy-runner`, with no wrapper.
It runs the post-generation scripts of the recipe at build time, as the
machine build does. No start runs them again. It installs no kernel, no
network configuration, and no vsock guest unit.
`/opt/ciproxy-image` gets only the files that a launch uses:
`boat-run.sh`, `boat-start.sh`, `measure.py`, and `metadata-smoke.py`.
The directory also keeps the Apache-2.0 [LICENSE](LICENSE).
The script also adds the `sandbox-workload` account that receives each
delivered registration. It installs `ciproxy-boat-start.service` and
`ciproxy-boat-guest.service` disabled, so a restore alone starts no guest.

## The locale of Boat's controller

Then [boat-provision.sh](boat-provision.sh) runs
[boat-build-locale.sh](boat-build-locale.sh). The machine build does not
run this script.

Boat's `/home/user/.profile` sets `LC_ALL=en_US.UTF-8` for the commands of
Boat's controller. After a restore, Boat can run `locale-gen` while it
already accepts commands. Without arguments, `locale-gen` first removes the
locale archive and each locale folder in `/usr/lib/locale`, except
`C.utf8`. Then it generates the configured locales again. Without other
locale data, the shell of a command in that gap prints a `setlocale`
warning on stderr, also when the command succeeds. The pinned Boat client
of the overflow backend rejects each helper reply with text on stderr. So
the warning stops the start of a runner.

The script writes the `en_US.UTF-8` data to its own folder,
`/usr/local/lib/ciproxy/locales/en_US.utf8`. `locale-gen` does not change
this folder. The script also installs
`/etc/profile.d/ciproxy-boat-locale.sh`, which a login shell reads before
`/home/user/.profile`. When `HOME` is `/home/user`, the `LOCPATH` that
this profile exports holds `/usr/local/lib/ciproxy/locales`. So a command
of Boat's controller finds the `en_US.UTF-8` data also while `locale-gen`
runs. Each other account, such as the runner account, keeps its locale
environment.

## The NVM profile of Boat's base

Last, `boat-provision.sh` runs [boat-build-nvm.py](boat-build-nvm.py). The
machine build does not run this script.

The first step keeps Boat's `/etc/profile.d/nvm.sh`, which each login
shell reads through `/etc/profile`. Its line 31 is
`__ascii_nvm_bin="$(__ascii_nvm_path 2>/dev/null)"`. The lookup returns 1
when it finds no default NVM version, as for the runner account. The
runner entry runs `set -eo pipefail` before it reads `/etc/profile`. So
without a guard, that line ends the entry with exit status 1 before the
listener starts.

The script appends `|| __ascii_nvm_bin=''` to that one assignment. A
failed lookup then leaves the variable empty, and the profile takes its
fallback as before. A found version still goes first in `PATH`. A second
run changes nothing. When the profile is missing, is a link, or has no
known lookup line, the script prints a warning and leaves the file
unchanged. The build does not fail on that warning.

## The runner stderr of a Boat job

The guest drops the stderr of the runner process. So in Boat mode, the
runner entry first sends its stderr to a journal helper:

- [boat-runner-entry.sh](boat-runner-entry.sh) starts
  [boat-runner-stderr.py](boat-runner-stderr.py) beside it, and sends its
  own stderr there before any profile runs. It sets `BASH_ENV` to the
  error hook, and then runs the unchanged common entry.
- [boat-runner-errors.sh](boat-runner-errors.sh) is the error hook. It
  first unsets `BASH_ENV`, so the listener and the job do not get it. For
  a failing command of the entry or its profiles, it prints only the file
  name, the line, and the exit status.
- `boat-runner-stderr.py` reads all of the stderr, so the runner never
  waits on it. It writes at most 16 KiB to the journal, with the
  identifier `ciproxy-boat-runner-stderr`, and then one truncation notice.
  A line longer than 4 KiB also ends the output with that notice.
- The helper removes the registration value from its own environment, and
  gives the journal writer no environment. It replaces a whole line with
  `[redacted]` when the line holds the registration value or a value
  decoded from it. It does the same for a credential word or a long opaque
  value. In each other line, it keeps the text before the first `://` and
  replaces the rest with `://[redacted]`. The scheme stays. The URL suffix
  does not reach the journal. It turns the constant missing-registration
  error into `[ciproxy] runner entry error: JIT registration required`.
- When the helper or the journal fails, the runner goes on, and its
  stderr is dropped.

## Reports

After the provisioners, the build writes its reports under
`/opt/ciproxy-build/reports`:

- `software-report.json`, the upstream software report of the Boat build.
- `installed-packages.json`, each installed package and its version.
- `installed-inventory.json`, the same packages, and whether the software
  report exists. It gives `apt_pin_differences`, each installed package
  whose version differs from its APT pin. The
  [README](README.md#inputs-and-build) gives its fields. It also gives the
  records that the build made of Boat's base system. A record that does
  not exist is `null`:
  - `provider_docker_gid`, the group ID of Boat's `docker` group.
  - `provider_directory_owners`, the owners of the four directories.
  - `provider_developer_environment`, each command, setting, and package
    group that the cleanup removed or kept, with its warnings.
- `inventory-differences.json`, each version difference from the
  reference inventory, and the same `apt_pin_differences`. It says when
  the software report is missing.
- `changed-services.json`, the running services whose program or unit file
  the build changed. Docker and containerd are always in this list.

The build also writes that service list to
`/etc/ciproxy/boat-restart-services` for each launch.
A version difference or a pin difference does not fail the build. Record
both with the release evidence.
