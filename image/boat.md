# Boat image preparation and smoke

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
The wrapper uses the provider's resolver file and does not start the
image's network services. The Firecracker network configuration does not
change Boat's addresses, routes, or DNS.
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
