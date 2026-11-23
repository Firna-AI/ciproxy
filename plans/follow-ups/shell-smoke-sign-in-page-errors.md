# Shell smoke sign-in cases report page errors

- **Severity:** Medium. **Effort:** Small. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-23, from the branch `calummoore/copying-llm-xioi`, a
  problem outside the scope that the full shell smoke run showed.
- **Location:** `ts/app/scripts/browser-smoke.ts`, the shell smoke.
- **Reason:** The failure is on `origin/main` too, so it is older than the
  branch that found it.

- **Terms:**
  - **Shell smoke:** `ts/app/scripts/browser-smoke.ts`. It drives headless
    Chrome against the fixture harness `ts/app/scripts/harness-server.ts`
    at 1440, 720, and 390 pixels.
- **Background:** Each case of the shell smoke fails when the page emits a
  JavaScript error or a console error.
- **Problem:** On 23 November 2026, 45 of 297 shell smoke checks failed with
  "The page emitted a JavaScript or console error" and the recorded event
  `pageerror`. The failed cases are `auth-expired`, `late-add-sign-in`,
  `late-first-sign-in`, `late-multiple-claims`, `root-access-recovery`,
  `root-explicit-workspace`, `root-install-completed`,
  `root-install-github`, `root-install-github-empty`,
  `root-install-handled`, `root-install-progress`, `root-install-viewer`,
  `root-reinstall-open`, `root-removed-old-claim`, and
  `root-removed-old-outcome`, at each width. The cases
  `root-install-completed`, `auth-expired`, `late-first-sign-in`, and
  `root-access-recovery` also failed at each width on a clean export of
  `origin/main` at commit `aa1d2b23`. The error text itself was not
  captured. The run used Google Chrome at `/usr/bin/google-chrome` and
  `playwright-core` from `ts/node_modules`.
- **Impact if no action:** The shell smoke cannot pass, so it cannot prove
  a change to the sign-in and root pages.
- **Root question:** Why do the sign-in and root pages emit a page error in
  the shell smoke?
- **Options:**
  - **A.** Capture the `pageerror` message in the smoke output, then fix
    its cause in the app or the harness. Effort: Small.
  - **B.** Do A, and make the smoke print the message of each page error
    by default. Effort: Small.
- **Left out:** Ignoring page errors in these cases hides real defects.
- **Wider fix:** B. Each later page error then names its cause.
- **Previous examples:** None found.
- **Recommendation:** B. The smoke then explains its own failures.
- **Action:** fix after merge, apply B, because the failure is older than
  this branch and is outside its scope.
