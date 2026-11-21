# Boat smoke

These checks start sandboxes from the Boat snapshot that the
[Boat build](boat.md#boat-shared-snapshot) saved.
Run each smoke below with `large`, as the full image requires.
Use `--trial` with `create` on a free trial to request its two-hour limit.
The default lifetime is six hours.
Keep the returned sandbox ID and process ID in the release evidence.

## Restore without a registration

This check needs no GitHub registration and contacts no GitHub service.

```sh
python3 image/boat.py create --size large \
  --share-file .context/ciproxy-image-share.json
python3 image/boat.py status RESTORE_SANDBOX_ID
python3 image/boat-locale-smoke.py RESTORE_SANDBOX_ID
python3 image/boat.py metadata RESTORE_SANDBOX_ID \
  --baseline .context/rootfs-metadata-before.json
python3 image/boat.py run RESTORE_SANDBOX_ID
python3 image/boat.py status RESTORE_SANDBOX_ID --process LAUNCH_PROCESS_ID
```

Repeat `status` until the state is `ready`, `idle`, or `running`. Then run
the locale check at once, before `metadata`, because Boat can still
generate its locales again at that time. The check runs
`/usr/bin/locale charmap` through Boat's controller, with the shell
settings of Boat's `user` account. That program prints the codeset of the
locale that the command inherits, and it changes no locale itself. The
command must exit with 0, print the `UTF-8` codeset, and leave stderr
empty. Then the check exits with 0 and prints this line:

```json
{"codeset": "UTF-8", "exit_code": 0, "stderr_empty": true}
```

Otherwise the check prints one fixed message on stderr and exits with 1.
It never prints text from the Boat reply. Require a pass before `run`.
The [Boat build steps](boat-build-steps.md#the-locale-of-boats-controller)
say why a locale warning stops the start of a runner. Boat's locale setup
can end before the check runs. So the
[Boat build check](boat-build-checks.md#the-locale-of-boats-controller)
also checks the locale without a locale archive. It runs the same command,
and it shows that the check rejects the result with `LANG=C` and no
`LC_*` setting.

The metadata comparison runs after the restore and before a launch.
It prints `matches_baseline` and each difference as smoke evidence.
Confirm that `matches_baseline` is `true` after each restore.
It adds no build gate. Investigate a difference before using the image.
Use `--file RELATIVE_PATH` in both metadata commands to check another file.
For an ACL proof, choose a file with a populated ACL attribute.
A `null` ACL records its absence and does not prove that the ACL stays.
The metadata script needs Python with Linux extended-attribute support.

Before `run`, confirm with the Boat command API that both units are
disabled:

```sh
systemctl is-enabled ciproxy-boat-start.service ciproxy-boat-guest.service
```

`run` without a registration file runs the launch with no path.
Require a zero exit of the launch process. The launch returns before the
start unit ends, so wait until the start unit has left its `activating`
state. Then run these commands in the sandbox with the Boat command API:

```sh
sudo systemctl show ciproxy-boat-start.service -p User -p ControlGroup \
  -p ActiveState -p SubState -p Result -p ExecMainStatus
sudo journalctl --unit ciproxy-boat-start.service --no-pager
sudo systemctl show ciproxy-boat-guest.service \
  -p ActiveState -p SubState -p Result -p ExecMainStatus
sudo journalctl --unit ciproxy-boat-guest.service --no-pager
sudo test ! -e /run/ciproxy/registration.json
sudo test ! -e /opt/ciproxy-build
sudo test ! -e /tmp/ciproxy-image
sudo test ! -e /etc/apt/apt.conf.d/90ciproxy-build
sudo test ! -e /etc/apt/preferences.d/ciproxy-software
sudo test -d /imagegeneration
sudo test -d /opt/post-generation
sudo systemctl is-active docker containerd ascii-agent
getent group docker
sudo test ! -e /etc/ciproxy/provider-docker-gid
docker version
sudo runuser --user runner -- docker run --rm hello-world
sudo systemctl start postgresql
pg_isready
sudo -u postgres psql --no-psqlrc --tuples-only --command 'SELECT 1'
```

Compare the Docker group number with `provider_docker_gid` in the saved
`installed-inventory.json`. Finalization removes the build-only record
from the sandbox.

The start unit must run as root in `system.slice`, outside `user.slice`.
Its journal must show each restart of a listed service, such as
`user@1000.service`, and then the start of the guest unit. A guest that
fails at once can make the start unit end with an error too. So read both
units and both journals.
The guest unit must show `ExecMainStatus=1`, and its journal must show
`registration file is missing`. The unit can report its start before that
error, so read it after the start. Docker and containerd must run the
versions on disk. The `docker` group must keep the group ID that the build
recorded from Boat's base system. Boat's user and the runner account must
both use Docker. The build files and the build apt settings must be
absent, and the two upstream folders must stay. Boat's controller must
still run commands and file transfers.

Then check the start of the runner entry, with no registration value.
The `runuser` command runs the installed entry as the guest does, with a
cleared environment. The journal commands before and after it share one
cursor file, so the second read shows only records that are newer than the
first read:

```sh
sudo grep -cF "|| __ascii_nvm_bin=''" /etc/profile.d/nvm.sh
sudo find /opt/actions-runner/_diag -name 'Runner_*.log'
sudo journalctl --identifier=ciproxy-boat-runner-stderr \
  --cursor-file=/tmp/ciproxy-runner-cursor --no-pager
sudo env -i /usr/sbin/runuser --user runner -- /usr/local/libexec/ciproxy-runner
sudo journalctl --identifier=ciproxy-boat-runner-stderr \
  --cursor-file=/tmp/ciproxy-runner-cursor --no-pager
sudo find /opt/actions-runner/_diag -name 'Runner_*.log'
```

The `grep` must print `1`, the count of the guarded NVM lookup. The entry
must exit with 1 at its registration check, because it has no
registration value. The entry sends its stderr to a journal helper, and
the exit of the entry does not show that the helper has written its
record. So repeat the second journal read about once a second, for up to
15 seconds, until the line below appears. Keep every record that these
reads show. The new records must show this line, and no error of
`nvm.sh`: `[ciproxy] runner entry error: JIT registration required`.
Compare the two file lists. No new `Runner_*.log` file must appear,
because the entry stops before it starts the listener. Another listener
call, such as `Runner.Listener --version`, can write its own diagnostic
files. Keep the journal lines and both file lists as smoke evidence.
Then delete the sandbox:

```sh
python3 image/boat.py delete RESTORE_SANDBOX_ID
```

## GitHub job smoke

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
    "registration": "ENCODED_JIT_REGISTRATION"
  },
  "exit_report_url": "https://callback.example/ACTUAL_SANDBOX_ROUTE",
  "exit_report_token": "TOKEN_FOR_THIS_SANDBOX_ONLY"
}
```

The upload uses a private temporary directory.
No credential enters Boat's command text or a saved snapshot.
The [Boat launch](boat.md#boat-launch) moves the file to a root-only path,
and the guest removes it before the runner starts.

Copy [smoke-workflow.yml](smoke-workflow.yml) to a private smoke repository.
It prints the account of the job. It checks the tools, Docker, and a
PostgreSQL start with `systemctl`.
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
The journal of `ciproxy-boat-start.service` shows the free bytes before
the guest starts.
While the sandbox exists, copy its bounded, redacted runner stderr with
the Boat command API:

```sh
sudo journalctl --identifier=ciproxy-boat-runner-stderr --no-pager
```

The exit callback deletes the sandbox, and the journal goes with it.
Read it at short intervals while the job runs. This read only copies the journal. It changes nothing in
the product.
A successful callback prints `runner exit report accepted`.
Verify the final Boat state as well. A successful job alone does not prove
that the callback deleted its sandbox.
If the control plane deletes it before status can read the logs, keep the
callback server's receipt and the provider deletion result as evidence.
Delete a remaining smoke sandbox with the explicit delete command.
`delete` uses the shared [Boat delete helper](boat_delete.py). The helper
takes the encoded request route and Boat's `X-Ascii-Confirm-Delete` header
from the same sandbox ID. The overflow smoke cleanup uses the same helper.
Change Boat deletion only in that helper. The
[delete confirmation check](checks.md#boat-delete-confirmation) needs no
Boat key or network:

```sh
python3 -B image/check-boat-delete.py
```

After a change to the helper, also run the local tests in the
[overflow guide](../crates/ciproxy-control-plane/overflow.md#local-tests).

Boat documents 51 GB for files on `small` and `default`.
Those sizes cannot hold the full software set.
Keep the full software set on `large` and measure its actual free job disk.
