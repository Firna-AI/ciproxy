# Boat overflow in the hosted runtime

This guide gives how the hosted runtime runs jobs on Boat when the own
machines are busy. It gives the configuration, the runtime guarantees, the
sandbox exit callback, and the live smoke checks. The
[crate README](README.md) gives the rest of the control plane.
[ciproxy-overflow](../ciproxy-overflow/README.md) owns sandbox delivery,
encrypted keys, costs, and cleanup. The
[overflow protocol](../../docs/protocol/overflow.md) gives the requirements.

## Configuration

`BOAT_SHARED_SNAPSHOT` is optional. It names the shared Boat snapshot or the
share code of the job image. The
[image guide](../../image/README.md#boat-shared-snapshot) creates the
snapshot and writes its share code to a private file. Keep that code out of
tracked files.

Use the snapshot of the same image release as `JOB_IMAGE_MANIFEST_PATH`.
The root bytes of that manifest size each new Boat root. The process does
not compare the two settings. Change them together.

An absent or empty value turns the Boat service off. Own machines still run
jobs. The dashboard API then reports `available: false`. Boat connection
changes then return HTTP 503, and the exit callback route is absent.

With a value, startup also builds the Cloud KMS client. `BOAT_KMS_KEY` must
then have the form `projects/P/locations/L/keyRings/R/cryptoKeys/K`.
Otherwise startup stops. The process gets KMS access tokens from the GKE
metadata server, so a key connection needs that server.

The runtime derives the sandbox mark and the callback URL from `PUBLIC_URL`.
It removes a trailing slash first. Boat sandboxes must reach
`PUBLIC_URL/hooks/boat/runner-exit` over HTTPS.

Keep the value set while a Boat sandbox can exist. Without it, the runtime
does no Boat cleanup and serves no exit callback. Boat then ends each
sandbox only at its lifetime. That is six hours, or two hours on a free
trial. The deploy script does not set this variable.
[Runtime setup](../../deploy/runtime.md#turn-on-the-runtime) gives the other
settings.

## Runtime guarantees

The routing snapshot of a workspace also holds the Boat account, its
deliveries, and the saved Boat registration identities. So the pure scaler
sees accepted Boat supply, and it starts no second runner for that demand.
The scaler applies own machines first, the
[overflow wait](../../docs/protocol/job-routing.md#the-overflow-wait), and
the Boat size rules. The runtime executes each Boat start through the
overflow service.

- The runtime saves a registration identity before it asks GitHub for a JIT
  registration. The identity holds the runner, delivery, organization,
  label, and scale-set IDs, the sizes, and the required root bytes.
- The JIT configuration goes only into the encrypted delivery input. The
  database never holds it in plaintext.
- An unfinished delivery resumes with the same IDs after a restart or an
  uncertain result.
- A registration without a live delivery is retired GitHub-first by its
  unique runner name, `ciproxy-<runner ID>`. The runtime then deletes its
  identity.
- An accepted delivery keeps its original request and its required root
  bytes, also after a new image release.
- A dispatched delivery keeps its fixed provider size. An unsent delivery
  can still select the current catalog size.
- A trusted GitHub job start marks the Boat runner busy. A completed job
  marks it exited, so it cannot take a second job while deletion waits.
- Idle retirement keeps a runner that a running job or waiting demand needs.
  Otherwise GitHub removes the registration first. A busy answer keeps the
  sandbox.
- After an owner removes the App, launched sandboxes stay until their
  runner exits or their saved lifetime ends. Missing job facts do not prove
  that a sandbox is idle. The runtime checks saved owner removal before and
  after GitHub retirement. A successful GitHub response does not permit
  early sandbox deletion after removal.
- App removal stops new Boat starts. The last organization removal retires
  the Boat key and holds it until the unfinished sandboxes end. Exit
  reports still delete their sandbox at once. App removal is rare, and the
  lifetime and spend limit bound its cost, as
  [Removing the App](../../docs/protocol/app-removal.md#removing-the-app)
  explains.
- Old idle supply cannot cover a replacement label with the same name.
  Retired labels and removed bindings cannot start an undispatched delivery.
- When Boat or its cost data fails, accepted Boat supply stays reserved and
  new Boat starts wait. Own-machine routing continues.

## Exit callback

`POST /hooks/boat/runner-exit` takes the runner-exit report of a sandbox.
The route uses no browser session. The guest sends
`GuestToAgent::RunnerExited` as JSON with `Authorization: Bearer TOKEN`.
Each sandbox has its own token, which works only for its own runner.

| Status | Meaning                                                             |
| ------ | ------------------------------------------------------------------- |
| 204    | The control plane accepts the report and starts sandbox deletion.   |
| 400    | The message is not `RunnerExited`, or its runner ID is not a UUID.  |
| 401    | The token is missing, wrong, already used, or for another runner.   |
| 503    | The request could not finish. The runtime retries saved cleanup.    |

Boat ends a sandbox at its lifetime when no report arrives.

## Live smoke checks

[scripts/overflow-smoke.py](scripts/overflow-smoke.py) runs three checks
against a deployed control plane. The checks start real GitHub jobs and
Boat sandboxes, and Boat bills the sandboxes. They need live credentials,
so they stay operator work. Each check turns overflow on with a 30-second
overflow wait.

- `overflow`: a hold job keeps the only own machine busy for 300 seconds.
  A job of the overflow label then waits and runs on Boat. The check matches
  the GitHub runner name to the saved delivery. It needs a successful job,
  an ended delivery, and Boat's confirmation of the deletion. The sandbox
  must start at least 30 seconds after the queue assignment of the job.
- `idle-delete`: the check cancels the overflow job after its sandbox exists
  and before a runner takes it. It needs an ended delivery and Boat's
  confirmation of the deletion, with no job on that runner. When the runner
  takes the job first, the check fails safely. Run it again.
- `spend-limit`: while the hold job runs, the check sets a one-cent spend
  limit. For 75 seconds, no delivery can dispatch, and the job must stay
  queued. The check then sets `smoke_limit_cents` again. The same job must
  then run on Boat, as in `overflow`.

### Before a check

1. Use a dedicated test workspace and a dedicated private GitHub repository.
   ciproxy runs only the jobs of
   [private repositories](../../docs/protocol/security.md#repository-access).
2. Run no other job in that workspace during a check. The cleanup deletes
   each Boat sandbox that the workspace starts during the check.
3. Install the test GitHub App for the owner of the repository. Give it
   access to that repository.
4. Connect exactly one own machine. Set its warm runners to zero.
5. Create two labels with the same vCPUs, memory, and disk. Choose a size
   so that the machine fits only one job at a time.
   - The hold label allows own machines only.
   - The overflow label allows own machines and Boat.
6. Copy the [smoke workflow](scripts/overflow-smoke-workflow.yml) to
   `.github/workflows/ciproxy-overflow-smoke.yml` on the default branch of
   the test repository. It needs no repository secret. Its run name is
   `ciproxy-smoke-<marker>`. Its one job holds a runner for `hold_seconds`.
7. Run the test control plane with `BOAT_SHARED_SNAPSHOT` and the matching
   `JOB_IMAGE_MANIFEST_PATH`, as [Configuration](#configuration) says.
8. Connect Boat in Setup or in Settings under Overflow, with a positive
   spend limit.
9. Get read access to the control-plane database for `psql`, for example
   through a Cloud SQL Auth Proxy.

The script stops before it dispatches a job when Boat is not connected or
not available. It also stops when the labels or the machine do not match
these steps.

### Credential file

Create a private JSON file outside the repository, with mode 0600. Do not
paste its contents into a log or a message.

| Key                 | Value                                                               |
| ------------------- | ------------------------------------------------------------------- |
| `base_url`          | The HTTPS origin of `PUBLIC_URL`, without `/api`.                   |
| `workspace_id`      | The ID of the test workspace.                                       |
| `session_cookie`    | The `__Host-ciproxy` cookie value of an Admin of that workspace.    |
| `csrf_token`        | The `csrf_token` of that session, from `GET /api/session`.          |
| `database_name`     | The name of the control-plane database, as the next paragraph says. |
| `github_token`      | A token with Actions read and write access to the test repository.  |
| `boat_api_key`      | The key of the connected Boat account.                              |
| `boat_wallet`       | Optional: `personal`, the default, or the Boat organization ID.     |
| `repository`        | The test repository as `owner/name`.                                |
| `workflow_id`       | `ciproxy-overflow-smoke.yml`, or the numeric ID of that workflow.   |
| `ref`               | The branch that holds the workflow file.                            |
| `hold_label`        | The name of the hold label.                                         |
| `overflow_label`    | The name of the overflow label.                                     |
| `smoke_limit_cents` | Positive US cents as a decimal string.                              |

The script gives `database_name` to `psql` as `PGDATABASE`. `psql` reads
that variable only as a database name, not as a connection URL. Export
`PGHOST`, `PGPORT`, and `PGUSER` for a role with read access before a check.
Keep a password in a private `PGPASSFILE`, not in the shell history.

Use the wallet of the connected key in `boat_wallet`. Set
`smoke_limit_cents` above the counted Boat spend of the current UTC month,
plus one more full-lifetime bound. Settings under Overflow shows the
confirmed and unconfirmed parts of that spend. The bound is six hours of
the hourly price of the label's Boat size, or two hours on a free trial,
plus the highest start fee. Each check adds to the counted spend.

### Commands

Run the commands from the repository root. Run one check at a time.

```sh
export CIPROXY_OVERFLOW_CREDENTIALS_FILE=/private/path/overflow-smoke.json
chmod 600 "$CIPROXY_OVERFLOW_CREDENTIALS_FILE"
python3 crates/ciproxy-control-plane/scripts/overflow-smoke.py --help
python3 crates/ciproxy-control-plane/scripts/overflow-smoke.py overflow
python3 crates/ciproxy-control-plane/scripts/overflow-smoke.py idle-delete
python3 crates/ciproxy-control-plane/scripts/overflow-smoke.py spend-limit
```

`--help` lists the three checks. It needs no credential file and no network.
Without `CIPROXY_OVERFLOW_CREDENTIALS_FILE`, a check prints a hint and exits
with status 2. A passed check prints `PASS:` lines and exits with status 0.
A failed check or a failed cleanup prints `FAIL:` lines and exits with
status 1. The script prints no key, cookie, token, registration, or
response body.

### Cleanup and recovery

The script cleans up after each check, also after a failure or Ctrl-C:

1. It turns off new Boat starts. It keeps the saved wait and limit.
2. It finds the run of each saved marker. It cancels each unfinished run
   and waits until the run completes.
3. It reads the durable start state again after cancellation completes.
   It waits until no new registration intent remains and each new delivery
   has ended. A missing sandbox ID does not prove that no sandbox exists.
4. It deletes each known Boat sandbox that the check started.
   It waits until Boat confirms each deletion.
5. It restores the original overflow switch, wait, and limit.

One 90-second deadline covers the whole cleanup. An unresolved start keeps
overflow off and prevents restoration. The script reports unresolved
delivery and runner IDs without credentials. When a step fails, it prints
`FAIL: <step> needs operator attention.` If a settings request fails, check
the current settings before another check. Do not press Ctrl-C again during
cleanup.

Each check records a new file under `.context/ciproxy-mvp-m15/live/`. It
saves `original_policy` there before it changes a setting. That policy holds
only `enabled`, `wait_seconds`, and `spend_limit_cents`. The file also lists
the `markers` and `runs` of the check. `delivery_baseline` and
`intent_baseline` identify records that existed before the check. The file
holds no credential. Git ignores it when the current directory is the
repository root.

When the script stopped before its cleanup ended, or printed a cleanup
`FAIL:` line, recover in this order:

1. Turn overflow off in Settings under Overflow. Keep the wait and the limit.
2. Open the newest file under `.context/ciproxy-mvp-m15/live/`.
   Keep its saved baselines. A new baseline can hide an earlier open start.
3. Find each run of the test repository with an ID from `runs`, or with the
   name `ciproxy-smoke-<marker>`. Cancel each unfinished run. Wait until each
   run completes.
4. Keep the control plane running with `BOAT_SHARED_SNAPSHOT` set. Its
   cleanup deletes each idle Boat sandbox that no job needs.
5. Check the Boat account for remaining ciproxy sandboxes of the test
   workspace. Boat ends each one at its lifetime at the latest.
6. In Settings under Overflow, restore the exact values of
   `original_policy`. A null `original_policy` means that the check changed
   no setting.

The GitHub CLI can list and cancel the runs:

```sh
gh run list --repo OWNER/NAME --workflow ciproxy-overflow-smoke.yml
gh run cancel RUN_ID --repo OWNER/NAME
```

A marker without a run can mean that GitHub did not accept that dispatch.
List the runs again after a minute before you decide.
