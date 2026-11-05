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
  the guest/runtime integration. Verify each job runtime token outside
  the VM: signature, issuer, expiry, and matching run and runner. Derive
  the full ref from the token's write scope. Shared writes also require
  the default branch and a trusted event. Keep administration credentials
  outside job VMs. Revoke sessions when jobs finish. Coordinate cache
  shutdown and file-handle closure before area unmount.
- **Impact if no action:** The agent falls back to direct GitHub caching
  and reports no local cache use. A separately configured cache can store
  data outside the area. If cache reclaim fails after it held space,
  `launches.rs` currently warns and starts the VM anyway. Occupied cache
  bytes can then block the disk promised to that VM.
- **Needs:** An integration owner that can change the agent, sidecar,
  guest, image startup, and public protocol when necessary. Follow
  `docs/protocol/cache.md` and the sidecar README's Agent integration
  section. Respect the resolved
  [image-reserve policy](agent-xfs-image-reserve.md).
  First complete the required live checks for a default-branch push,
  `pull_request_target`, and a job on a ciproxy runner. The full-ref
  failure policy still waits for the user. Do not invent that fallback.
- **Done when:** The service uses the area's cache directory. Its budget
  acknowledgement follows actual reclaim. On service failure, an owned
  cleanup path releases occupied space before VM use and preserves the
  required GitHub fallback. Removal closes cache handles before unmount.
  Real jobs prove cache use, scope isolation, fallback, and data placement.
  Failure-and-retry tests cover budget acknowledgement failure. The
  public agent still depends on no workspace crate except protocol.
