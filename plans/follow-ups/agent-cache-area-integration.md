# Own-machine cache and Docker need live proof

- **Severity:** Medium. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-05, from
  `conductor/ciproxy-mvp-milestone-9-agent`, host storage checks. Updated
  2026-11-06 from `conductor/ciproxy-mvp-milestone-13-e2e`, after its cache
  and Docker integration code.
- **Location:** `crates/ciproxy-agent/scripts/smoke-own-machines.py`,
  `crates/ciproxy-agent/scripts/own_cache.py`,
  `crates/ciproxy-agent/scripts/own_docker.py`,
  `crates/ciproxy-agent/scripts/smoke-host.sh`,
  `crates/ciproxy-agent/src/cache.rs`,
  `crates/ciproxy-agent/src/cache_service.rs`,
  `crates/ciproxy-agent/src/docker.rs`,
  `crates/ciproxy-cache-sidecar/src/cli.rs`,
  `crates/ciproxy-guest/src/handoff.rs`, and `image/docker-attach.sh`.
- **Reason:** The live checks need the merged change on the main preview
  control plane, a fresh disposable KVM host, and private credential files.

- **Terms:**
  - **Area:** The agent's mounted XFS filesystem at
    `/var/lib/ciproxy/area`.
  - **Machine cache:** `ciproxy-cache.service`, the host service that keeps
    the cache entries of the jobs on one customer machine.
  - **Budget acknowledgement:** The HTTP 204 answer of the machine cache to
    `PUT /budget`. It confirms the smaller budget before a VM starts or a
    Docker disk takes space.
  - **Docker lease:** The agent's durable record of the bytes that the
    Docker disk or disposable copy of one job reserves.
  - **Live check:** A run on a real KVM host with real GitHub Actions jobs.
    A `SKIP` result is not a live check.
- **Background:** The cache and Docker integration is implemented in code.
  No live check of it has run. The implementation environment had no KVM
  device, and its smoke command exited 77 before it read credentials.
  The code does these things:
  - The source-built runner Worker hook,
    `image/runner-patch/CiproxyAssignment.cs`, pauses before action setup.
    It sends the original `ACTIONS_RUNTIME_TOKEN`, cache URL, results URL,
    and effective cache permissions to the guest over
    `/run/ciproxy/assignment.sock`.
  - The guest adds the Docker bridge proxy URL and the preloaded Docker
    bytes. It sends `GuestToAgent::JobStarting` to the agent over vsock.
    The agent answers with `AgentToGuest::PrepareJob`. The guest answers
    with `GuestToAgent::JobPrepared` after its proxy binds. Then action
    setup continues. `crates/ciproxy-protocol/src/guest.rs` defines these
    messages.
  - These credentials stay in memory. They never enter hosted messages,
    logs, or durable agent state. The control plane gets only the runner
    and job IDs.
  - The agent calls the authenticated HTTP administration of the machine
    cache on `127.0.0.1:8229`. It reads the token file
    `/var/lib/ciproxy/cache-admin-token`. The old newline JSON socket is
    gone.
  - Agent install writes `ciproxy-cache.service`. The unit requires
    `var-lib-ciproxy-area.mount` and uses `/var/lib/ciproxy/area/cache`.
    VMs reach only its job listener, on TCP port 7887.
  - The unit omits `--capacity-bytes`, so the service starts with an
    unknown budget. It recovers its retained entries, writes no new
    archive, and evicts nothing until the agent sends `PUT /budget`. An
    explicit capacity, also zero, still reclaims entries at startup.
  - Each job gets its own session token and checked scope from
    `POST /sessions`. Recovery reuses a live session only for the exact
    registration. Close and fallback match the complete trusted
    assignment: organization, repository, run, job, and runner.
  - Before a VM starts or a Docker disk takes space, the agent sends the
    smaller cache budget. When administration fails, the agent stops the
    service, runs `ciproxy-cache-sidecar reclaim` on the cache directory,
    and starts the service again. When that release fails, the agent
    retires a new VM, and admission waits for repair. A Docker disk is
    then not attached, and its job uses the root disk.
  - The periodic budget refresh allocates no space. When it fails, the
    agent starts the service if necessary and tries again later. It
    does not run the offline cache clear for that failure.
  - A failed fallback or revoke call stops the service. This invalidates
    its sessions.
  - The hosted runtime sends each machine its Docker spare disk: the job
    disk that no set of feasible roots can use. The agent grants a Docker
    lease only from that spare disk. Active leases reduce root admission.
    A job without room keeps Docker on its root disk.
  - Only the first trusted default-branch writer of a repository keeps its
    Docker changes. Every other job gets a disposable disk. The guest
    mounts the disk with `image/docker-attach.sh` and checks its free bytes
    before Docker starts again.
  - Agent, guest, and cache tests cover these paths with mocks, local
    files, and local HTTP. They use no KVM host.
  - These decisions are settled. On 5 November 2026, the user chose signed
    token verification and fail-closed unknown scopes. An unknown job
    restores default-branch local entries, uses GitHub for other restores,
    saves privately, and gets a disposable Docker copy. Also on 5 November
    2026, the user chose that a Docker disk reserves its data plus the
    free disk of its label, only from spare disk. The images keep their
    own budget in the area, as the orchestrator chose on 5 November 2026.
- **Task:** Run each live requirement below on a fresh disposable KVM host
  against the main preview. Write a repeatable check for each requirement
  that has no script yet. Give each defect that a check finds its own fix
  or follow-up item.
  1. Actual KVM jobs: every scenario below runs real GitHub Actions jobs
     in Firecracker VMs on the registered machine.
  2. Cache hit: the `cache` scenario runs two jobs with the same key in two
     fresh VMs. `GET /report` must show the second job as
     `machine_handled` with at least one local entry.
  3. Scope isolation: the `cache_isolation` scenario saves a private entry
     on a non-default branch. A job of a second allowed repository and a
     job of another non-default branch must both miss it.
  4. Fallback: the `cache_fallback` scenario masks `ciproxy-cache.service`.
     The next job must restore the entry from GitHub and succeed. The
     script unmasks and starts the unit again in a `finally` block.
  5. Docker attach and space: the `sticky_docker` scenario needs a writer
     lease for the first default-branch job and the same image in the next
     fresh VM. It needs a disposable lease for `sticky_copy_ref`, and a
     later writer must keep the original image. Each attached disk length
     must equal its leased bytes. The `docker_no_fit` scenario needs zero
     spare disk, no lease, and a successful Docker job on the root disk.
     Both scenarios wait for the agent's `job_prepared_observed` journal
     event, which proves the preparation acknowledgement in a real VM.
  6. Data placement: `sticky_docker` reads its leases and disks inside the
     area. No script checks cache entries yet. Show that the cache entries
     go to `/var/lib/ciproxy/area/cache` on the area mount. Show that no
     cache data goes to the root filesystem outside the area.
  7. Budget failure recovery: no script checks it yet. Make the budget
     acknowledgement fail while cache files hold space. Show that the agent
     stops the service, reclaims the files, starts the service again, and
     starts the VM with its promised free disk. Show that the job uses
     GitHub or sees a miss. Show that a failed reclaim retires the start
     and blocks admission until repair. Show that a failed periodic
     refresh does not run the offline cache clear.
  8. Service restart: no script checks it yet. The `restart` scenario
     restarts the agent, not the machine cache. Save an entry, then restart
     `ciproxy-cache.service` during a running cache job. Show that the
     restarted service keeps that entry and writes no new archive before
     the agent sends its budget. Show that the running job succeeds and
     that the next job restores the saved entry from the machine cache.
     Show that the report hides the counts of the interrupted job and
     keeps the seven-day hit counts.
  9. File closure before unmount: no script checks it yet.
     `smoke-host.sh` proves that uninstall removes the area, its record,
     its mount unit, and the mount. It does not prove the stop order with
     an active machine cache. Run uninstall while the service is active
     and holds entries. Show from the journal that the service stopped
     before the area mount. Show that uninstall then removed the area.
- **Impact if no action:** The tests use mocks and local files. A real host
  can still break the systemd stop order, the guest drive switch and
  mount, the filesystem growth, the Docker restart, or the proxy readiness.
  The machine cache falls back to GitHub by design. So a broken machine
  cache can look like a normal cache miss, and customers lose the local
  cache without a visible error. An unproved scope boundary can let one
  branch or repository restore a private entry of another. An unproved
  budget recovery can start a VM without its promised free disk.
- **Needs:**
  - The merged change on the main preview, with the runtime turned on. The
    deploy workflow runs for each push to
    `calummoore/ci-proxy-load-balancer` and `main`. The pinned image
    release must include the guest program, the runner hook, and Docker.
  - Matching verified `ciproxy-agent` and `ciproxy-cache-sidecar` archives
    at `CIPROXY_RELEASE_SERVER_URL`, because install fetches both.
    `crates/ciproxy-agent/scripts/prepare-smoke-release.py` makes a draft
    GitHub release from them. Add `--publish` only to release them.
  - A fresh disposable x64 host with root, a usable `/dev/kvm`, systemd,
    cgroup v2, journalctl, `gh`, `jq`, `rg`, and Python 3. Reuse no agent
    state, image index, or VM record of an older installation.
  - Private files outside the repository. Each environment variable below
    holds a file path, never a credential:
    - `CIPROXY_OWN_SMOKE_INPUT` names the operator settings JSON. These
      scenarios need `public_url`, `workspace_id`, `machine_id`,
      `repository`, `ref`, `workflow`, `cache_isolation_repository`,
      `cache_writer_ref`, `cache_reader_ref`, `sticky_tier_ready`,
      `sticky_copy_ref`, and `docker_no_fit_ready`.
      `crates/ciproxy-agent/smoke-checks.md` gives the complete schema.
    - `CIPROXY_SMOKE_CREDENTIALS` names a JSON file with `github_token`
      and `session_cookie`, the private `__Host-ciproxy` dashboard value.
      `smoke-host.sh` also reads `install_token` from it.
      `prepare-smoke-release.py` reads `github_token` from it.
    - `CIPROXY_CACHE_ADMIN_TOKEN_FILE` names
      `/var/lib/ciproxy/cache-admin-token`, which agent install creates.
      The `cache` scenario reads it for `GET /report`.
  - The private allowed repository, a second allowed private repository,
    and a disallowed private repository. Each has
    `crates/ciproxy-agent/scripts/github-smoke-workflow.yml` as
    `.github/workflows/ciproxy-agent-smoke.yml`. Add two non-default
    branches for the cache writer and reader, and a non-default
    `sticky_copy_ref` branch, with the same workflow.
  - A workspace on Team or Business for `sticky_docker`. A separate
    disk-limited machine with zero spare disk for `docker_no_fit`.
  - Run the scripted checks as root from the repository root:

    ```sh
    export CIPROXY_OWN_SMOKE_INPUT=/private/own-smoke-input.json
    export CIPROXY_SMOKE_CREDENTIALS=/private/own-smoke-credentials.json
    export CIPROXY_CACHE_ADMIN_TOKEN_FILE=/var/lib/ciproxy/cache-admin-token
    for scenario in cache cache_isolation cache_fallback sticky_docker; do
      python3 crates/ciproxy-agent/scripts/smoke-own-machines.py \
        --scenario "$scenario"
    done
    python3 crates/ciproxy-agent/scripts/smoke-own-machines.py \
      --scenario docker_no_fit
    ```

    Run `docker_no_fit` on the disk-limited machine. Run the install,
    repair, and uninstall smoke on a host with `/var/lib` on ext4:

    ```sh
    export CIPROXY_SMOKE_REPO=owner/private-agent-smoke
    export CIPROXY_SMOKE_REF=main
    export CIPROXY_AGENT_URL=wss://PREVIEW_HOST/agent/
    export CIPROXY_SMOKE_CREDENTIALS=/private/own-smoke-credentials.json
    export CIPROXY_SMOKE_BINARY=target/debug/ciproxy-agent
    export CIPROXY_SMOKE_INTEGRATION_READY=1
    bash crates/ciproxy-agent/scripts/smoke-host.sh
    ```

  - Each script exits 0 on success, 77 with `SKIP` for a missing
    prerequisite, and 1 on a failure. Evidence goes under
    `.context/ciproxy-mvp/own-smoke-*` and
    `.context/ciproxy-mvp/host-smoke-*`. It holds run IDs, job UUIDs, VM
    IDs, and counts, but no token or cookie.
- **Done when:**
  - Each of the nine live requirements has a passing live check on a fresh
    disposable KVM host against the main preview.
  - Each requirement that had no script has a repeatable check under
    `crates/ciproxy-agent/scripts/`, described in
    `crates/ciproxy-agent/smoke-checks.md`.
  - The saved evidence holds no token, cookie, or other credential.
  - Each fix keeps `ciproxy-agent` free of workspace crates other than
    `ciproxy-protocol`.
- **Related:**
  - [Image project quota](agent-xfs-image-project-quota.md) asks only
    whether the kernel also enforces the image budget. Such a quota does
    not reclaim cache space. This task does not wait for that answer.
  - [Interrupted uninstall](agent-interrupted-install-uninstall.md) fixes
    uninstall of an incomplete installation. Both use guarded area
    removal. The unmount check of this task uses a complete installation.
