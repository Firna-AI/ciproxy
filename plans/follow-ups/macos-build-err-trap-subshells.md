# The macOS build reports a failure from a subshell

- **Severity:** Low. **Likelihood:** Rare. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-23, from `calummoore/h4-publish-and-xcode-fetch-gomg`,
  review round 3, the wider fix of the finding on a doubled failure line.
- **Location:** `image/macos/build.sh`, the `set -E` line and the `ERR`
  trap near the top; `image/macos/build-lib.sh`, `guest_ssh` and other
  helpers that use command substitution.
- **Reason:** The change alters the failure report of every phase, and only
  a build run on a Mac shows each phase.

- **Terms:**
  - **ERR trap:** the bash handler that `build.sh` sets to print
    "build.sh: phase X failed at line N (exit S)". `set -E` makes subshells
    and command substitutions inherit it.
- **Background:** The publish branch removed the subshell around the fetch
  phase, so a failed fetch prints one failure line. A test in
  `image/macos/check-r2-xcode.py` checks that. Other subshells stay. For
  example, `guest_ssh` runs `$(guest_ip 5)`.
- **Problem:** When a command fails inside a command substitution, the
  inherited trap prints a "phase failed" line with an inner line number,
  also when the script then goes on, and again in the parent.
- **Impact if no action:** A failed phase can print two failure lines, or a
  failure line for a step that the script survives, so the operator reads
  the wrong line.
- **Root question:** How does a failed phase report itself exactly once?
- **Options:**
  - **A.** Print only in the top-level shell. Save `$?` first:
    `trap 'status=$?; (( BASH_SUBSHELL == 0 )) && echo "build.sh: phase
    $phase failed at line $LINENO (exit $status)" >&2' ERR`. Effort: Small.
  - **B.** Drop `set -E` and the ERR trap, and report from the EXIT trap
    when the status is not 0. The report loses the line number. Effort:
    Small.
- **Left out:** `set +E` inside a subshell does not remove a trap that the
  subshell already inherited.
- **Wider fix:** A. It covers every subshell in every phase.
- **Previous examples:** The fetch phase subshell, removed on the publish
  branch.
- **Recommendation:** A. It keeps the line number.
- **Action:** fix after merge, apply A, because the change needs a build run
  on a Mac to check.
