# The older macOS image scripts fail shellcheck

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-23, from `calummoore/h4-publish-and-xcode-fetch-gomg`,
  review round 1, a problem outside the scope that the check of the
  shellcheck finding raised.
- **Location:** `image/macos/build-lib.sh`, `image/macos/build.sh`.
- **Reason:** The scripts belong to the macOS image build of PR #113, and
  that branch was still in work. So the publish branch changed only its own
  scripts.

- **Terms:**
  - **shellcheck:** a static checker for shell scripts.
    `image/macos/.shellcheckrc` makes it follow the sourced files of the
    folder.
- **Background:** The README of `image/macos/` gives a shellcheck command
  for `publish.sh`, `fetch.sh`, and `r2.sh`, and those three pass. No gate
  runs shellcheck.
- **Problem:** `shellcheck image/macos/*.sh` reports 8 notes in two older
  scripts. `build-lib.sh` has SC2148, SC2154 for `vm_tool` and `work_dir`,
  SC2120, SC2119, and SC2029. `build.sh` has SC2016 twice.
- **Impact if no action:** A lint of the whole folder fails, so a person
  cannot tell a new problem from the known notes.
- **Root question:** How does one lint command cover every macOS image
  script and pass when nothing is wrong?
- **Options:**
  - **A.** Clear the 8 notes: add `# shellcheck shell=bash` to
    `build-lib.sh`, add a directive for each intended pattern, and fix the
    `vm_start` argument pattern. Then the README names the whole folder.
    Effort: Small.
  - **B.** Do A, and run shellcheck on `image/macos/*.sh` in the gate, as
    `plans/follow-ups/image-checks-in-gate.md` proposes for the image
    checks. A new gate needs the user. Effort: Medium.
- **Left out:** A severity filter such as `-S warning` still fails on
  SC2148 and hides real notes.
- **Wider fix:** B. Only a gate keeps the folder clean over time.
- **Previous examples:** None found.
- **Recommendation:** A. It is small and needs no new gate.
- **Action:** fix after merge, apply A, because the scripts belong to the
  open macOS image build branch.
