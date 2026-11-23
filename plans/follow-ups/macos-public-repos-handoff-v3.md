# macOS runners need the public-repository refusal

- **Severity:** High. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-23, from `calummoore/support-mac-os-kq1u`, the end of
  the plan `plans/macos-own-machines.md`.
- **Location:** `image/macos/` (the image), `crates/ciproxy-guest`,
  `image/runner-patch/`, `image/macos/package-guest.sh`.
- **Reason:** The contract comes from the public repositories branch
  `calummoore/support-public-repos-sepq` (plan
  `plans/public-repositories.md`, Milestone 5), which was not on `main`
  when this item was written.

- **Background:**
  - Public-pool jobs from a fork are refused before their first step. The
    contract: `RefusalReason::Fork`, `JobDecision::Refused`,
    `HANDOFF_PROTOCOL_VERSION` 3, the Worker probe
    `ciproxy-handoff-protocol-3`, and `RunnerSpec.pool`.
  - The agent side is shared code, so it also covers Macs.
  - The Worker patch gets a second patch point that completes a refused job
    as Failed with no step. The macOS Worker is built by the same
    `prepare-source.py`.
  - The published macOS image `…-standard-v2` has handoff protocol 2.
  - Public macOS jobs need the placement "Your machines and Boat", because
    Boat has no macOS.
- **Task:**
  1. After that branch merges, check that both patch points apply to the
     `osx-arm64` Worker build, and run the runner-patch tests on a Mac.
  2. Check that the macOS guest speaks handoff protocol 3 over vsock.
  3. Build, publish, and pin a new standard image.
  4. On the test Mac, run a public-pool job from a fork (refused before its
     first step) and one from the same repository (runs).
  5. State the macOS placement rule on the site's macOS image page.
- **Impact if no action:** A Mac with the old image cannot run public-pool
  jobs safely.
- **Action:** post-merge task, after the public repositories branch.
