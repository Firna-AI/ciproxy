# Each build wires its guest pre-start apart

- **Severity:** Low. **Effort:** Medium. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-23, from the branch `calummoore/swap-support-3dll`,
  review findings 4 and 9, the shared wider fix.
- **Source:** Review.
- **Category:** Test.
- **Location:** `image/ciproxy-guest.service`,
  `image/ciproxy-boat-guest.service`, `image/boat-start.sh`,
  `image/provision.sh`, `image/_checks_/boat_fixture.py`, and
  `image/_checks_/export_fixture.py`.
- **Reason:** The branch made each fixture check the swap start, and one
  shared pre-start setup changes both launch paths.

- **Terms:**
  - **Pre-start step:** a step that must finish before the guest starts
    the runner, such as the zram swap unit `ciproxy-swap.service`.
  - **Machine image:** the Linux image that a Firecracker job VM boots.
    The Boat snapshot is the same software on Boat's own system.
- **Background:**
  - The machine image enables `ciproxy-swap.service`, ordered before
    `ciproxy-guest.service`. The Boat launch starts the same unit from
    `image/boat-start.sh` before `ciproxy-boat-guest.service`.
  - The Boat launch check and the export check each verify their own
    path. `plans/follow-ups/boat-guest-registration-path-coverage.md`
    records a similar gap in the Boat fixture.
  - The export check once passed when the swap unit had no `Before=` rule
    or the enablement link was missing from `rootfs.ext4`. It now checks
    both on the released disk.
- **Problem:** Each new pre-start step needs two separate wirings and two
  separate checks.
- **Impact if no action:** A later pre-start step can reach one build and
  miss the other.
- **Root question:** How does a pre-start step reach both guest units from
  one place?
- **Options:**
  - **A.** Keep the two wirings and their checks. Effort: Small.
  - **B.** Give both guest units one shared pre-start setup, such as a
    `ciproxy-guest-prestart.target` that each guest unit requires, and test
    it in the launch and export fixtures. Effort: Medium.
  - **C.** Let the guest program run each pre-start step before the
    runner. Cost: the guest gets a system setup duty. Effort: Large.
  - **D.** Use `ExecStartPre=-/usr/local/libexec/ciproxy-swap` in both guest
    units. The directive requests the helper, orders it before the guest,
    and waits for it. This reverses the separate unit design. Swap setup
    then waits for Docker. Effort: Medium.
- **Left out:** A text match on the scripts, because the `testing` skill
  asks for a test of the behaviour through the real entry point.
- **Wider fix:** B.
- **Previous examples:**
  `plans/follow-ups/boat-guest-registration-path-coverage.md`.
- **Recommendation:** B. One place then gives each step to both builds.
- **Action:** fix after merge, apply B, because it changes both launch
  paths.
