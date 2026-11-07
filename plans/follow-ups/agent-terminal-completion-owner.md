# Terminal completion has separate local owners

- **Severity:** Low. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-07, from
  `conductor/follow-ups-agent-egress-install-network`, review round 1,
  findings 2 and 4 and the pre-existing early-exit identity retention.
- **Source:** Review.
- **Category:** Code structure.
- **Location:** `crates/ciproxy-agent/src/controller.rs`,
  `crates/ciproxy-agent/src/network_lifecycle.rs`,
  `crates/ciproxy-agent/src/application.rs`,
  `crates/ciproxy-agent/src/runtime.rs`, and
  `crates/ciproxy-agent/src/host_preparation.rs`.
- **Reason:** The direct startup and daemon wakeup corrections are complete.
  One completion owner requires a wider Agent and runtime change that can
  wait until after merge.

- **Terms:**
  - **Terminal completion:** A removed machine has no remaining VM or owned
    network resource after checked cleanup.
  - **Identity finalization:** The agent deletes its saved machine identity
    after all required local cleanup succeeds.
  - **Inspection flag:** The in-memory `network_inventory_valid` value that
    permits the passive completion check to succeed.
- **Background:** The Apache-2.0 agent runs job VMs on a customer machine.
  Its local terminal entry points are startup, daemon removal, and uninstall.
  Empty Removed startup now checks networks before requiring an identity.
  Worker completion events now wake disconnected removal after cleanup.
  These direct defects are corrected without changing the passive check.
- **Problem:** Checked cleanup and identity finalization still have separate
  owners in the three entry points. The early successful startup exit still
  retains any saved revoked identity. That retention existed before the
  network ownership correction and remains outside its direct fixes.
- **Impact if no action:** A later caller can omit a cleanup or finalization
  step. The existing early exit can retain an unnecessary revoked credential
  on disk, although that credential no longer grants control-plane access.
- **Root question:** How can every local terminal entry point use the same
  checked cleanup and identity finalization rules?
- **Options:**
  - **A.** Align each entry point separately and correct the early exit.
    Keep the passive check and inspection flag. Each caller must maintain
    its own cleanup and finalization sequence. Effort: Small.
  - **B.** Give Controller one terminal completion operation that drives
    checked network cleanup when no runner remains. Make startup, the
    daemon, and uninstall use that owner. Let the owner finalize identity
    only after the cleanup required by that entry point succeeds. Replace
    the passive inspection flag with that operation. Effort: Medium.
  - **C.** Persist explicit cleanup and finalization phases and make all
    entry points use their shared reader. This adds durable writes and
    compatibility work for an additional stored format. Effort: Medium.
- **Left out:** Trusting a missing identity or defaulting inspection to
  complete cannot prove that old orphan networks are absent. Disabling the
  service hides incomplete cleanup. A new timer duplicates existing retry
  scheduling. Keeping the revoked identity indefinitely fails the goal.
- **Wider fix:** B. One completion owner keeps checked cleanup and identity
  finalization in the same sequence for every local entry point.
- **Previous examples:** In `crates/ciproxy-agent/src/application.rs`, the
  early successful Run exit precedes identity clearing after `daemon.run`.
  Commit `436efe9a7dd5e0c6a133a7cc6e4032595b750806` already had that flow,
  before this branch's startup and network fixes. The startup operation
  preserves it.
- **Recommendation:** B. It gives every local entry point one completion
  contract without another durable phase record.
- **Action:** fix after merge, apply B, because the direct startup and wakeup
  defects are corrected and the wider Agent/runtime coordination can wait.

## Current entry points and requirements

- `crates/ciproxy-agent/src/controller.rs` defines
  `Agent::finish_removed_startup` and the passive `Agent::finished` check.
  The startup operation runs network-only cleanup for an empty Removed
  machine. It does not clear a retained identity.
- `crates/ciproxy-agent/src/network_lifecycle.rs` owns checked network
  cleanup and pending ownership. It sets the inspection flag after
  successful host inspection.
- `crates/ciproxy-agent/src/application.rs` returns early when startup
  completes locally. Its later successful daemon return clears identity
  through Store.
- `crates/ciproxy-agent/src/runtime.rs` waits for typed worker completion
  events before checking `finished` again.
  `crates/ciproxy-agent/src/runtime_worker.rs` publishes replies, errors,
  and completion without replies.
  `crates/ciproxy-agent/src/runtime_reports.rs` owns the existing ten-second
  report and reconciliation schedule.
- `crates/ciproxy-agent/src/host_preparation.rs` defines `Uninstall::start`
  and `finish`. Uninstall preserves identity until area, tools, and service
  cleanup succeeds. The common owner must preserve that order.
- `crates/ciproxy-agent/src/state.rs` and
  `crates/ciproxy-agent/src/persistence.rs` define current state and its
  local reader. Existing saved state must remain loadable.
- [Agent requirements](../../docs/protocol/agent.md) require running jobs to
  finish during removal and require the removed agent to stop afterward.
  Uninstall finishes running jobs before removing its owned area.
- [Security requirements](../../docs/protocol/security.md) keep machine
  credentials root-only and revoke their access at removal.

## Work requirements

- Apply one completion contract to startup, daemon removal, and uninstall.
- Keep removal independent of control-plane access or a saved credential.
- Preserve active jobs and terminal Removed mode across cleanup waits.
- Keep RunnerExited delivery independent of network host commands.
- Preserve independent report scheduling and delivery.
- Retry with existing timers. Add no new timer.
- Remove the retained identity on successful terminal startup completion.
- Preserve identity when uninstall area, tools, or service cleanup fails.
- Keep current state compatibility and the Apache-2.0 dependency boundary.
- Test the production entry points with fake state and host boundaries.
  No raw credential is needed for this work.

The work is complete when all local terminal entry points share checked
cleanup and identity finalization, with the required uninstall ordering.
