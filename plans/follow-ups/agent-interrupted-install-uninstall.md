# Uninstall rejects an incomplete installation

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-05, from
  `conductor/ciproxy-mvp-milestone-9-agent`, review finding 13.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-agent/src/application.rs`,
  `crates/ciproxy-agent/src/area.rs`, and
  `crates/ciproxy-agent/src/host_preparation.rs:94`.
- **Reason:** This Rare Medium defect has manual recovery and changes no
  stored format. Its correction costs the same after merge.

- **Terms:**
  - **Area:** The owned XFS filesystem that holds images and job data.
  - **Creating:** The recorded phase before filesystem preparation ends.
  - **Ready:** The recorded phase after the filesystem identity check.
  - **Removing:** The recorded phase after removal starts.
- **Background:** Installation prepares the area before registration.
  It records Creating before formatting. It records Ready after mounting
  and measured growth. An interrupted installation can leave Creating with
  no recorded VM. A completed area's mount can also stop. VM startup saves
  ownership before starting its systemd unit.
- **Problem:** The uninstall CLI permits direct cleanup only when the
  area record is absent or Removing. Creating and unmounted Ready instead
  enter runtime verification. Verification rejects them before guarded
  removal runs. It also rejects a mounted area whose reflink check fails.
- **Impact if no action:** Uninstall leaves the area file, ownership
  record, and tools. A failed mount can leave an enabled mount unit that
  retries at each boot. The error suggests restoring a backup even when
  no ownership data is damaged. Reinstallation repeats a persistent
  formatting or mount fault. Manual root-level removal is the only exit.
- **Root question:** How does an operator remove a machine with no
  recorded VM, whatever state host preparation left its area in?
- **Options:**
  - **A.** Route an empty machine directly through existing guarded
    removal in the CLI. Test the lower area boundary. Those tests cannot
    protect the CLI routing that caused this defect. Effort: Small.
  - **B.** Move the routing and preparation steps into the existing
    `Uninstall` service. Keep the drain loop in the CLI. Route a machine
    with no recorded VM directly to guarded removal. Route a recorded VM
    through verification, image recovery, restoration, and drain.
    Test both routes through that service. Effort: Small.
- **Left out:** Reinstallation cannot fix a persistent formatting or
  mount fault. Relaxing runtime verification lets image recovery write
  under an unmounted mountpoint. A separate cleanup command duplicates
  removal and adds an unnecessary CLI surface. Install rollback misses
  power loss and deletes data that repair should preserve. Manual removal
  instructions leave uninstall broken. A new forwarding controller adds
  a layer that B does not need. Moving the full event loop adds work
  without removing another cause of this defect.
- **Wider fix:** B. The default sends an empty machine to guarded removal.
  It requires no runtime checks. Later preparation checks cannot block it.
- **Previous examples:** None found in the targeted lifecycle history
  and repository search.
- **Related:**
  - [Image project quota](agent-xfs-image-project-quota.md) can add a quota
    step to preparation. This removal fix does not depend on that answer.
  - [Cache integration](agent-cache-area-integration.md) must close cache
    handles before unmount. Both routes still use guarded area removal.
  - [Network ownership](agent-network-cleanup-ownership.md) preserves
    cleanup ownership. A machine with a recorded VM keeps its drain route.
- **Recommendation:** B. It puts the safe default and its regression test
  in the existing service.
- **Action:** fix after merge, apply B, because the defect is Rare and
  Medium, has manual recovery, and changes no stored format.
- **Review outcome, 5 November 2026:** The orchestrator selected B in the
  first review of pull request 12. It is the existing-service routing
  formerly labelled C in this record. Only the letter changed.

## Implementation requirements

`FinalUninstall` already holds images, preparation, service, store, and
agent boundaries. Give it the area-files boundary and one start method.
Preserve the ownership-record error message when inspection fails.
With no VM record, call guarded removal, uninstall the service, and clear
state. Do not verify, recover images, or register the machine.
With a VM record, verify before image recovery, restoration, and removal.
Keep the CLI drain loop and final `finish` call.

Guarded removal already checks ownership, XFS identity, unmount, and loop
users. It permits a never-formatted Creating file. It skips the filesystem
identity check only for that owned blank file. Do not weaken those checks.

Extend `src/_tests_/host_preparation_tests.rs` with the actual service
entry point. An empty Creating machine must remove without verification.
A machine with a VM record must verify before removal. Add a lower area
case for the never-formatted Creating file in `area_cleanup_tests.rs`.
The existing `uninstall_unmounts_before_erasing_only_its_recorded_area`
already covers an unmounted Ready area. Preserve that case.

Run targeted agent tests and the complete gate with `DATABASE_URL` set.
Keep the agent Apache-2.0 and dependent only on `ciproxy-protocol` inside
the workspace.
