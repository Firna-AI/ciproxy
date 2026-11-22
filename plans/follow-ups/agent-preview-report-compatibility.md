# Main preview rejects agent 0.1.4 reports

- **Severity:** High. **Likelihood:** Common. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-22, from `conductor/boat-callback-diagnosis`, item 1
  of a read-only compatibility check of the main preview. The check ran
  while the two held Boat smokes were blocked.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-protocol/src/reports.rs:44-58` and
  `crates/ciproxy-gateway/src/transport.rs:194-204`, at commit `c5cd00be`.
- **Reason:** The recommended B adds a requirement at the agent boundary
  that no protocol doc states, so the user must choose.

- **Terms:**
  - **Agent 0.1.4:** the `ciproxy-agent` release with the tag
    `ciproxy-agent-v0.1.4`, at commit `b751ef5f`.
  - **Gateway:** the part of the control plane that accepts agent
    WebSockets at `/agent/`. Its code is in `crates/ciproxy-gateway`.
  - **Report:** the machine state in each `Register`, `Hello`, and `Report`
    message. Its hardware section describes the processor, memory, disks,
    and VM support.
  - **Error close:** a close that the gateway starts, with the WebSocket
    code 1008 and a fixed reason, such as `[gateway] invalid message`.
  - **Main preview:** the hosted test deployment that
    `.github/workflows/deploy-main-preview.yml` deploys on each push to
    `main` and to one other branch.
- **Background:**
  - Agent 0.1.4 sends the hardware fields `processor`, `threads`,
    `memory_bytes`, `kvm`, and `disks`, and no platform. See
    `crates/ciproxy-protocol/src/reports.rs:43-55` and
    `crates/ciproxy-agent/src/hardware.rs:69-74` at `b751ef5f`.
  - It runs only as root on x64 Linux (`hardware.rs:27-28`). It sends
    `Hello` with a full report on each connection
    (`crates/ciproxy-agent/src/gateway.rs:55-63`).
  - Commit `b528de33` made `hardware.platform` and
    `hardware.virtualization` required, and it removed `hardware.kvm`. It
    reached `main` in merge `6ab68943` on 22 November 2026 at 05:11:30 UTC.
    Each earlier `main` build, up to `3760f9bf`, has the `kvm` shape.
  - At `c5cd00be`, `HardwareReport` derives `Deserialize` with no default
    (`reports.rs:44-58`). The enums `Platform` and `Virtualization` have no
    default value and no catch-all value
    (`crates/ciproxy-protocol/src/platform.rs:12-20` and `:37-51`).
  - No protocol type uses `deny_unknown_fields`. So the decode ignores the
    old `kvm` key and fails on the missing `platform` key.
  - The protocol version is still 1
    (`crates/ciproxy-protocol/src/version.rs:14`). The gateway accepts the
    current and the previous number
    (`crates/ciproxy-gateway/src/versions.rs:7-16`). So no version number
    marks the change.
  - The gateway decodes the first message before it checks the credential
    (`crates/ciproxy-gateway/src/transport.rs:102-115`). Each decode error
    becomes `Error::InvalidMessage` and an error close, and the decode text
    is dropped (`transport.rs:194-204`).
  - No test decodes the hardware section of agent 0.1.4 at `c5cd00be`.
    Commit `b528de33` rewrote the legacy test JSON of
    `crates/ciproxy-protocol/src/_tests_/wire/agent_tests.rs` to the new
    shape.
  - Agent 0.1.4 treats only the reason `[gateway] authentication failed` as
    a revocation (`crates/ciproxy-agent/src/connection.rs:118-132`). It logs
    each other close as "control plane unavailable; preserve VMs and
    reconnect", with `[agent] internal error` (`gateway.rs:43-46`).
  - It then retries with a delay that doubles up to 60 seconds. It keeps
    its identity and its VMs.
  - Commit `15805cb0` added the migration `202611221200_platform`. It
    replaces `machines.kvm_available` with `machines.virtualization`. Its
    message says that the gateway now rejects a `Hello` whose platform
    differs from the registered platform.
  - The control plane runs pending migrations at startup
    (`crates/ciproxy-store-pg/src/factory.rs:27-28`). Nobody read the
    preview database. So nobody knows whether this migration ran there.
  - Observed on 22 November 2026:
    - The dashboard of the main preview reported build `c5cd00be`. It
      showed the smoke machine of the Firna-AI workspace offline.
    - On that machine, the `ciproxy-agent` service was active, and SSH
      worked. The machine kept its Firna-AI identity.
    - From 10:52:40 to 11:03:43 UTC, the agent logged its unavailable
      warning every 60.23 to 60.52 seconds. So each try failed in under one
      second.
    - The machine had shown ready on 21 November 2026, against a `main`
      build with the `kvm` shape.
    - The Google login failed at the first `kubectl` call, and all Google
      calls stopped. So nobody read the control-plane logs, the database,
      or the deploy times.
    - The two held Boat smokes did not start, and no paid sandbox started.
      Four sandboxes of the task are deleted, and the last two slots are
      unused.
  - Evidence limits:
    - The source proves the rejection. No log proves it as the live cause,
      because the agent log hides the reason.
    - An HTTP, TLS, or DNS failure at the upgrade gives the same agent log.
      Even then, agent 0.1.4 cannot become ready on `c5cd00be`.
    - `--version` on the machine printed 0.1.4. `main` also has
      `version = "0.1.4"` in `Cargo.toml:7`, and an agent built from `main`
      sends the new shape. Nobody checked the hash of the machine's binary.
    - The build comes from the dashboard. Nobody checked an image digest.
    - The local tags end at `ciproxy-agent-v0.1.4`. Nobody checked the
      download site for a newer release.
- **Problem:** Build `c5cd00be` cannot decode any report of agent 0.1.4,
  because the report has no `platform`. The gateway ends each connection
  with an error close before it checks the credential. So a machine on
  agent 0.1.4 cannot come online against this build.
- **Impact if no action:** The smoke machine stays offline, and the two
  held Boat smokes stay blocked. Each machine on agent 0.1.4 fails the same
  way against the main preview.
- **Root question:** How do the main preview and the agent on the smoke
  machine get back to one wire shape that both accept?
- **Options:**
  - **A.** Hold the two smokes until a later change restores one wire
    shape. Then check the machine again. No such change is known. A costs
    no branch and no paid slot, but the smokes stay blocked with no end
    date. Effort: Small.
  - **B.** In a separate workspace, make the control plane accept the 0.1.4
    hardware section. Agent 0.1.4 stays the published release.
    - Decode a missing `platform` as `linux_x64`, because agent 0.1.4 runs
      only on x64 Linux.
    - Derive a missing `virtualization` from `kvm`: true gives `available`,
      and false gives `kvm_unavailable`.
    - Add a regression test that decodes the released 0.1.4 `Hello` fields
      with the current types. The test covers those fields only.
    - State the new requirement in the protocol doc: the control plane
      accepts the hardware section of agent 0.1.4.
    - Check the platform that the migration gave existing machines, because
      the gateway rejects a platform that differs from the registered one.
    - A merge into `main` deploys B to the main preview. The decode can go
      once each machine runs a newer release.
    - Effort: Medium.
  - **C.** Publish an agent release newer than 0.1.4 from `main`.
    Explicitly change the task requirement that the smoke machine runs
    agent 0.1.4 to that release.
    - `main` still says 0.1.4 in `Cargo.toml:7`, so C needs a version
      change first.
    - Agent 0.1.4 checks for a new release every hour, also while it is
      offline (`crates/ciproxy-agent/src/maintenance.rs:42-48`). So the
      machine updates itself.
    - The release is public, and it ships each agent change of `main`
      since 0.1.4.
    - A control plane built before merge `6ab68943` then rejects the new
      report, because it requires `kvm`.
    - Effort: Medium.
  - **D.** Roll the main preview back to a build before merge `6ab68943`,
    such as `3760f9bf`.
    - A deploy by hand needs Google Cloud access, which failed on
      22 November 2026.
    - If the platform migration ran on the preview database, the old code
      needs that migration reverted first. Nobody read its `down.sql`.
    - The next push to `main` or to the other deploy branch deploys the new
      code again. The preview is shared, so each preview user loses the
      changes after that build meanwhile.
    - Effort: Large.
- **Left out:**
  - Install an unreleased `main` build on the machine by hand. It fails the
    goal, because the smoke would test a binary that no release holds. That
    binary also reports 0.1.4.
  - Revert merges `6ab68943` and `2e7ccf17` on `main`. B is better on every
    point. It keeps agent 0.1.4 working with a smaller change, and it needs
    no new migration to undo the schema change.
  - Raise the protocol version and decode version 1 as the old shape.
    Agents built from `main` also send version 1, so the gateway would
    still need the decode of B. So B is better on every point.
- **Wider fix:** None among these options. Each option fixes this one
  change of the wire shape.
- **Previous examples:** None found of a break without a version change.
  Commit `f3e31c3f` kept older statistics sections decodable with a default
  status. Commit `cf622fcb` raised `HANDOFF_PROTOCOL_VERSION` to 2 for a
  breaking guest change.
- **Related:**
  - `plans/follow-ups/boat-live-completion-acceptance.md`, work that waits:
    its two smoke runs end with this machine Ready on agent 0.1.4, which
    this rejection prevents. The failed Google login also blocks them.
  - `plans/follow-ups/install-failure-messages.md`, same failure path: the
    agent turns a close reason into `[agent] internal error`. So its log
    did not name this cause.
  - `plans/follow-ups/gateway-error-close-logging.md`, same failure path:
    the gateway logs each error close only at the debug level. So its
    default hosted log does not name this cause either.
  - Options C and D conflict. A control plane that D rolls back rejects the
    report of the release that C publishes.
- **Recommendation:** B. It brings back each machine on agent 0.1.4 with
  one small decode. It keeps 0.1.4 as the published release, and it needs
  no new release and no shared rollback.
- **Action:** ask the user, because B adds a requirement at the agent
  boundary that no protocol doc states. The control plane then accepts the
  hardware section of agent 0.1.4.
