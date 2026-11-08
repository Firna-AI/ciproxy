# KVM and GitHub smoke checks

Use a disposable private repository with the staging GitHub App installed.
Copy [the workflow fixture](scripts/github-smoke-workflow.yml) into that
repository as `.github/workflows/ciproxy-agent-smoke.yml`.
Push it to the branch that the scripts will dispatch.
The staging gateway must route its `ciproxy` jobs to this machine.
It must supply a verified image and keep a warm runner for the warm check.
The image must include the guest program and Docker.
Create a private credential JSON file with `github_token`.
Set `CIPROXY_SMOKE_CREDENTIALS` to that file.
The scripts disable shell tracing and never print the token.
Run each check as root on the registered KVM machine:

```sh
export CIPROXY_SMOKE_CREDENTIALS=/private/ciproxy-smoke-credentials.json
export CIPROXY_SMOKE_REPO=owner/private-agent-smoke
export CIPROXY_SMOKE_REF=main
crates/ciproxy-agent/scripts/smoke-docker.sh
crates/ciproxy-agent/scripts/smoke-warm.sh
crates/ciproxy-agent/scripts/smoke-restart.sh
```

The [ext4 host smoke](host-preparation.md#ext4-and-kvm-smoke) also checks
first install, repair, two real reflink jobs, and uninstall.

Optional inputs are `CIPROXY_SMOKE_WORKFLOW` and `CIPROXY_SMOKE_STATE`.
The restart check restarts the agent during a real Docker job.
It checks the same VM process and a successful job result.
It also requires a fresh accepted JobStarted observation from the new agent.
The script filters structured JSON journal events by service invocation.
The observation contains only the generated local VM ID.
The warm check proves that the runner was ready before dispatch.
It snapshots every VM ID before dispatch, including Starting VMs.
The replacement warm VM must have an ID absent from that snapshot.
An active machine that never reaches warm readiness fails the check.
The smoke host also needs journalctl.
Each check waits for the used VM and disks to disappear.
Evidence stays under `.context/ciproxy-mvp/`.
Missing resources produce `SKIP` and exit 77. Failed checks exit 1.

## Complete own-machine runtime check

The new runtime and image use one current format only.
Do not reuse agent state, image indexes, or VM records from an older test
installation. Use a fresh disposable host. Uninstall the older agent after
its jobs finish before you reset its files or prepare the host again.

Run the selected image's [guest network smoke](../../image/checks.md#guest-network)
before the registered-job scenarios. It checks public DNS and HTTPS in a
private guest copy before any runner starts and needs no GitHub credential.

Copy the updated workflow fixture to the allowed private repository.
Copy it to a second private repository that the runner group disallows.
The workflow's cache scenario uses the pinned official `actions/cache`.
Set these environment variables to private file paths:

```sh
export CIPROXY_OWN_SMOKE_INPUT=/private/own-smoke-input.json
export CIPROXY_SMOKE_CREDENTIALS=/private/own-smoke-credentials.json
export CIPROXY_CACHE_ADMIN_TOKEN_FILE=/var/lib/ciproxy/cache-admin-token
python3 crates/ciproxy-agent/scripts/smoke-own-machines.py
```

The input JSON contains only operator settings:

```json
{
  "public_url": "https://preview.app.ciproxy.dev",
  "workspace_id": "WORKSPACE_UUID",
  "machine_id": "MACHINE_UUID",
  "repository": "owner/allowed-private-repo",
  "disallowed_repository": "owner/disallowed-private-repo",
  "ref": "main",
  "workflow": "ciproxy-agent-smoke.yml",
  "parallel_jobs": 2,
  "cache_isolation_repository": "owner/second-allowed-private-repo",
  "cache_writer_ref": "cache-writer-branch",
  "cache_reader_ref": "cache-reader-branch",
  "item_79_policy_verified": true,
  "sticky_tier_ready": true,
  "sticky_copy_ref": "untrusted-docker-copy-branch",
  "docker_no_fit_ready": false,
  "deadline_seconds": 600,
  "queued_observation_seconds": 60
}
```

The credential JSON contains `github_token` and `session_cookie`.
`session_cookie` is the private `__Host-ciproxy` value from the dashboard.
The older first-install host smoke also reads `install_token` from this
same private credential file. No raw token environment variable is used.

The runtime check gets CSRF from `/api/session`.
It sends the exact public origin on every dashboard mutation.
It never follows redirects with those credentials.
Evidence contains only run IDs, job UUIDs, VM IDs, and observed counts.
It stays under `.context/ciproxy-mvp/own-smoke-*`.

The scenarios are warm, slots, drain, offline, cache, cache_isolation,
cache_fallback, sticky_docker, docker_no_fit, restart, disallowed, and remove.
Use `--scenario NAME` to run one scenario.
Remove runs last because it revokes the disposable machine.
It requires `item_79_policy_verified` after the policy is selected and checked.
It checks a job already observed as running.
It does not prove the unresolved assignment/removal race of item 79.
The isolation check needs two non-default branches with the fixture.
The reader branch cannot be the writer branch or the default branch.
It also needs another allowed private repository on the same machine.
The check writes a unique branch cache and requires misses across both
repository and branch boundaries.
The fallback check masks the host service temporarily.
It requires a successful GitHub restore while that service is unavailable.
The script restores the cache unit in a `finally` block.
A slot check on a minimum host can SKIP when two default jobs do not fit.
Sticky Docker persistence uses the selected spare-disk reservation policy.
Its KVM attachment and filesystem checks still need live proof.
A missing live prerequisite reports SKIP with exit 77.
A skipped scenario never counts as live success.

## Matching host releases

First install also fetches `ciproxy-cache-sidecar` from the selected release
endpoint. Earlier agent release assets do not contain that executable.
Prepare matching verified archives before the live host check.
Set `CIPROXY_RELEASE_SERVER_URL` to the trusted disposable release endpoint.
Build the release binaries with the GNU target and glibc check from CI.
Package both with `scripts/cli-release.sh package`.

The operator can prepare a GitHub release from those verified assets:

```sh
python3 crates/ciproxy-agent/scripts/prepare-smoke-release.py \
  --repository owner/release-repository --tag ciproxy-agent-vVERSION \
  --assets .context/agent-release
```

This creates a draft. Add `--publish` only when the operator intends to
make the verified release available to the release server.
The script reads `github_token` from `CIPROXY_SMOKE_CREDENTIALS`.
It validates each archive against its adjacent SHA-256 file before upload.
It never prints authentication headers or private response bodies.
No release was created or published in this implementation session.

## Sticky Docker proof

The `sticky_docker` scenario requires Team or Business and enough spare disk.
The default branch and `sticky_copy_ref` must both contain the workflow.
The copy ref must be a non-default branch.
The script requires a persistent host lease for the first default writer.
It checks that the next fresh VM restores that repository's Docker image.
It then requires a disposable host lease for the copy branch.
A later default writer must retain the original image and reject the copy's
new image. Host lease records supply repository, access mode, and bytes.
The script verifies the actual backing length against those reserved bytes.
Workflow output supplies the Docker image marker only.

Run `docker_no_fit` separately on a prepared disk-limited machine.
Set `docker_no_fit_ready` only after its trusted host state reports zero
spare bytes. The script waits for the guest readiness observation.
It then requires no host lease and a successful Docker job on the root disk.
This case does not need a new Docker budget setting or a host ledger.
These scenarios have not run here because KVM is unavailable.

## Docker disk recovery

Use a disposable registered KVM machine with the current agent and image.
Install the workflow fixture in its allowed private repository.
The workspace needs Team or Business and enough spare Docker disk.
Set the two private input paths from the runtime check above.
Set `sticky_tier_ready` to true after you confirm that spare disk fits.
Keep other workflows of this repository stopped during this check.

```sh
sudo --preserve-env=CIPROXY_OWN_SMOKE_INPUT,CIPROXY_SMOKE_CREDENTIALS \
  python3 crates/ciproxy-agent/scripts/smoke-docker-recovery.py
```

The script creates a repository disk through a trusted job.
Its default `--fault guest` mode installs a temporary systemd override.
A private `dumpe2fs` wrapper returns the real admitted free-space result.
Before returning it, the wrapper corrupts that job's writer disk once.
It checks the repository, writer lease, hard-link identity, and byte length.
The actual guest mount must fail and report root Docker fallback.
The detached lease must retain its bytes across an agent restart.
The VM must keep the same process through that restart.
The script removes its service override in a `finally` block.
Its separate `--fault offline` mode corrupts the unused disk while the
agent is stopped. It checks host filesystem rejection before attachment.
Both modes require a successful Docker job on its root.
The following trusted job must create a new persistent disk.
A later job must restore that new disk's Docker image.
Evidence stays under `.context/ciproxy-mvp/m25/`.
Missing root, KVM, or private inputs reports `SKIP` with exit 77.
The host also needs Python 3, systemd, `dumpe2fs`, and `/usr/bin/env`.
It must use the standard agent service at `/usr/local/bin/ciproxy-agent`.
These live checks have not run here because KVM is unavailable.

## Cache output firewall

The [cache firewall proof](cache-egress-smoke.md) checks the installed
service user, private upstream blocks, DNS, cache jobs, and failed starts.
It also requires a real host reboot before it reports PASS.
