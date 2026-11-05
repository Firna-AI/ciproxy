# Uninstall rejects an incomplete installation

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-05, from
  `conductor/ciproxy-mvp-milestone-9-agent`, review finding 13.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-agent/src/application.rs:217`,
  `crates/ciproxy-agent/src/area.rs:81`, and
  `crates/ciproxy-agent/src/host_preparation.rs:94`.
- **Reason:** This Rare Medium defect changes no stored format. Its safe
  lifecycle correction costs the same after merge.

- **Terms:**
  - **Area:** The owned XFS filesystem that holds images and job data.
  - **Creating:** The recorded phase before filesystem preparation ends.
  - **Ready:** The recorded phase after the filesystem identity check.
  - **Removing:** The recorded phase after removal starts.
- **Background:** Installation prepares the area before registration.
  It records Creating before formatting. It records Ready before starting
  the mount. An interrupted installation can leave either phase with no
  recorded VM. VM startup saves ownership before starting its systemd unit.
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
  - **C.** Move the routing and preparation steps into the existing
    `Uninstall` service. Keep the drain loop in the CLI. Route a machine
    with no recorded VM directly to guarded removal. Route a recorded VM
    through verification, image recovery, restoration, and drain.
    Test both routes through that service. Effort: Small.
  - **B.** Move routing and the drain loop into one lifecycle controller.
    This adds controlled-clock and guest-channel tests without removing
    another case of this defect. Effort: Medium.
- **Left out:** Reinstallation cannot fix a persistent formatting or
  mount fault. Relaxing runtime verification lets image recovery write
  under an unmounted mountpoint. A separate cleanup command duplicates
  removal and adds an unnecessary CLI surface. Install rollback misses
  power loss and deletes data that repair should preserve. Manual removal
  instructions leave uninstall broken. A new forwarding controller adds
  a layer that C does not need.
- **Wider fix:** C, changed from B after the independent check. The
  default sends an empty machine to guarded removal without runtime
  checks. Later preparation checks cannot block that route.
- **Previous examples:** None found in the targeted lifecycle history
  and repository search.
- **Related:**
  - [Image reserve](agent-xfs-image-reserve.md) can add preparation checks.
    This removal fix does not depend on that answer.
  - [Cache integration](agent-cache-area-integration.md) must close cache
    handles before unmount. Both routes still use guarded area removal.
  - [Network ownership](agent-network-cleanup-ownership.md) preserves
    cleanup ownership. A machine with a recorded VM keeps its drain route.
- **Recommendation:** C, changed from B after the independent check.
  It puts the safe default and its regression test in the existing service.
- **Action:** fix after merge, apply C, because the defect is Rare and
  Medium, leaves owned files that manual removal clears, and costs the
  same to fix later.

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
