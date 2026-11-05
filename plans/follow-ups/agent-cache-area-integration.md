# Cache storage and control need integration

- **Severity:** Medium. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-05, from
  `conductor/ciproxy-mvp-milestone-9-agent`, host storage checks.
- **Location:** `crates/ciproxy-agent/src/cache.rs`,
  `crates/ciproxy-agent/src/launches.rs`,
  `crates/ciproxy-cache-sidecar/src/cli.rs`,
  `crates/ciproxy-cache-sidecar/src/session.rs`, guest startup, and runtime
  composition.
- **Reason:** The host task prepares the data path, but complete cache
  wiring crosses the agent, sidecar, guest, and runtime boundaries.

- **Terms:**
  - **Area:** The agent's mounted XFS filesystem at
    `/var/lib/ciproxy/area`.
  - **Machine cache:** The persistent cache service for jobs on one
    customer machine.
  - **Budget acknowledgement:** Confirmation that the cache reclaimed
    enough space before a VM starts.
- **Background:** Host preparation creates `/var/lib/ciproxy/area/cache`.
  The agent currently expects newline JSON at
  `/run/ciproxy-cache/control.sock`. The merged sidecar instead exposes
  authenticated HTTP administration and trusted runtime-session
  registration. Its data location comes from `--cache-dir`. Creating the
  directory alone does not start or configure that service.
- **Task:** Wire the machine service to the prepared cache directory and
  persistent area dependency. Align its control interface with the agent.
  Supply trusted job facts, runtime credentials, and permissions through
  the guest/runtime integration. Use the machine service's existing token
  verification outside the VM. It checks signature, issuer, validity,
  repository, run, runner, and optional job identity. Use its returned
  checked scope for Docker persistence. Only the first job with the exact
  default ref, a trusted event, and no foreign head keeps its changes.
  Other jobs and unknown jobs get disposable copies. Keep administration
  credentials outside job VMs. Revoke sessions when jobs finish.
  Coordinate cache shutdown and file-handle closure before area unmount.
- **Impact if no action:** The agent falls back to direct GitHub caching
  and reports no local cache use. A separately configured cache can store
  data outside the area. If cache reclaim fails after it held space,
  `launches.rs` currently warns and starts the VM anyway. Occupied cache
  bytes can then block the disk promised to that VM.
- **Needs:** An integration owner that can change the agent, sidecar,
  guest, image startup, and public protocol when necessary. Follow
  `docs/protocol/cache.md` and the sidecar README's Agent integration
  section. The cache uses the job disk that no VM holds, as "Disk" in
  `docs/protocol/job-routing.md` says. The images use a separate budget in
  the area, which the orchestrator chose on 5 November 2026. That budget
  waits for no answer. The
  [project quota item](agent-xfs-image-project-quota.md) asks only whether
  the kernel also enforces it. Such a quota does not reclaim cache space.
  The user chose signed token verification and fail-closed unknown
  scopes on 5 November 2026. Unknown jobs restore default-branch local
  entries, use GitHub for other restores, save privately, and receive a
  disposable Docker copy. This policy is settled.
- **Done when:** The service uses the area's cache directory. Its budget
  acknowledgement follows actual reclaim. On service failure, an owned
  cleanup path releases occupied space before VM use and preserves the
  required GitHub fallback. Removal closes cache handles before unmount.
  Real jobs prove cache use, scope isolation, fallback, and data placement.
  Failure-and-retry tests cover budget acknowledgement failure. The
  public agent still depends on no workspace crate except protocol.

## Missing own-machine handoff

`crates/ciproxy-protocol/src/guest.rs` sends only runner and job IDs in
`GuestToAgent::JobStarted`. It carries no original `ACTIONS_RUNTIME_TOKEN`
or original cache and results URLs. `RunnerSpec` also carries no effective
cache permissions. The guest emits Ready and Exited only. It has no
trusted job-start pause, credential handoff, or readiness acknowledgement.
A shell hook does not receive the original runner runtime credentials.
Reading job-controlled environment values would break the trust boundary.

The machine service's authenticated `POST /sessions` requires those
credentials and URLs with trusted assignment facts, effective permissions,
and the VM's proxy URL. It returns a session token and checked or unknown
scope. The agent must never invent these inputs or send the runtime token
to the control plane. It must keep them out of logs and persistent state.

The agent starts Firecracker with `--no-api` and two fixed drives. It has
no safe job-start drive switch or persistent repository Docker store.
`AgentToGuest::DockerDisk` names an attachment but supplies no preparation
or attach acknowledgement. The guest currently ignores that message.
Choosing a persistent backing file after GitHub assigns a warm runner
therefore also needs coordinated guest and VMM work before action steps.

Extend or supply the shared handoff through
`crates/ciproxy-protocol/src/guest.rs`,
`crates/ciproxy-protocol/src/runners.rs`,
`crates/ciproxy-guest/src/boundary.rs`,
`crates/ciproxy-guest/src/service.rs`, and `image/runner-patch` with their
owners before claiming own-machine cache registration or checked-scope
Docker persistence. Those sources are outside the agent workspace's
assigned file scope. No change to those sources was made by the
host-preparation implementation.
