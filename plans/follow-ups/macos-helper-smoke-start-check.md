# The macOS helper smoke test ignores a failed start

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-22, from the branch
  `calummoore/macos-s-vm-memory-whole-mib`, round 2 of its review.
- **Location:** `crates/ciproxy-agent/scripts/macos-helper-smoke.sh`, the
  line `running "${id}" || true` after `launch "${id}" cold`.
- **Reason:** The script is a manual Mac check, and the memory fix of
  that branch does not touch it.

- **Terms:**
  - **Helper smoke test:** `macos-helper-smoke.sh SIGNED_AGENT BUNDLE
    [GUEST_BINARY]`. It runs on a Mac, starts one VM helper with launchd,
    and checks its control socket.
- **Background:** `running ID` waits up to 60 s for the answer
  `OK running` from the control socket. The script calls it as
  `running "${id}" || true`. Without the optional `GUEST_BINARY`, the
  script skips the guest checks, stops the helper, and passes when the
  helper job unloads.
- **Problem:** A VM that never reaches `running` does not fail the
  script when no guest binary is given.
- **Impact if no action:** A manual Mac run can report success for a VM
  that did not start.
- **Root question:** How does the smoke test fail when the VM does not
  start?
- **Options:**
  - **A.** Replace `|| true` with a `FAIL` message and `exit 1`, after it
    prints the helper log. Effort: Small.
  - **B.** As A, and also test a label memory that is not a whole MiB,
    such as 8,000,000,000 bytes, in `vm.json`. Effort: Small.
- **Left out:** Only one script exists for this check, so there is no
  third place for the fix.
- **Wider fix:** None. The `testing` skill already requires a check to
  fail on the defect that it covers.
- **Previous examples:** None found.
- **Recommendation:** B. It also covers the memory rule that failed the
  first real start.
- **Action:** fix after merge, apply B.
