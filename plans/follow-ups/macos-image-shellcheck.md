# The older macOS image scripts fail shellcheck

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-23, from `calummoore/h4-publish-and-xcode-fetch-gomg`,
  review round 1, a problem outside the scope that the check of the
  shellcheck finding raised.
- **Location:** `image/macos/build-lib.sh`, `image/macos/build.sh`,
  `image/macos/configure-guest.sh`.
- **Reason:** The `vm_start` change and the guest script can be checked
  only on a Mac with a VM, so the publish branch changed only its own
  scripts.

- **Terms:**
  - **shellcheck:** a static checker for shell scripts.
    `image/macos/.shellcheckrc` makes it follow the sourced files of the
    folder.
- **Background:** The README of `image/macos/` gives a shellcheck command
  for `publish.sh`, `fetch.sh`, and `r2.sh`, and those three pass. No gate
  runs shellcheck.
- **Problem:** `shellcheck image/macos/*.sh` exits 1. On 23 November 2026,
  shellcheck 0.9.0 reported 10 notes in three older scripts:
  - `build-lib.sh`: SC2148, SC2154 for `work_dir` and `vm_tool`, SC2120 and
    SC2119 for the `vm_start` arguments, and SC2029.
  - `build.sh`: SC2016 twice, and SC2034 for the unused loop variable
    `wait`.
  - `configure-guest.sh`: SC2174 for `mkdir -p -m`.

  The list can change when the scripts change, so the goal is the exit
  status, not the list.
- **Impact if no action:** A lint of the whole folder fails, so a person
  cannot tell a new problem from the known notes.
- **Root question:** How does one lint command cover every macOS image
  script and pass when nothing is wrong?
- **Options:**
  - **A.** Run `shellcheck image/macos/*.sh` first, and clear every note
    that it reports: add `# shellcheck shell=bash` to `build-lib.sh`, add a
    directive for each intended pattern, rename the unused variable, and fix
    the `vm_start` argument pattern. Done when the command exits 0. Then the
    README names the whole folder. Effort: Small.
  - **B.** Do A, and run shellcheck on `image/macos/*.sh` in the gate, as
    `plans/follow-ups/image-checks-in-gate.md` proposes for the image
    checks. A new gate needs the user. Effort: Medium.
- **Left out:** A severity filter such as `-S warning` still fails on
  SC2148 and hides real notes.
- **Wider fix:** B. Only a gate keeps the folder clean over time.
- **Previous examples:** None found.
- **Recommendation:** A. It is small and needs no new gate.
- **Action:** fix after merge, apply A, because parts of the change can be
  checked only on a Mac with a VM.
