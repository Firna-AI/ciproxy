# Readiness option A overstates its report coverage

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-21, from `conductor/boat-locale-fix`, final review
  round 4, finding 10. Its check corrected the severity from Medium.
- **Location:** `plans/follow-ups/boat-launched-runner-readiness.md:43-44`,
  `:105-116`, and `:158-159`, and `crates/ciproxy-guest/src/boat.rs:52-60`.
- **Reason:** The branch that found it used both of its actual fix rounds,
  and this Low docs finding is not critical.

- **Terms:**
  - **Readiness item:** the pending user decision
    `plans/follow-ups/boat-launched-runner-readiness.md`, about Boat starts
    that fail and send no exit report.
  - **Guest:** the ciproxy program in the image that starts the runner of
    one job and reports the end of that runner.
  - **Registration file:** the root-only `/run/ciproxy/registration.json`.
    It holds the runner configuration and the callback address and token.
  - **Exit report:** the guest's HTTPS callback with the sandbox token.
- **Background:**
  - This is not a runtime defect. The record of a pending proposal claims
    more than that proposal can do.
  - Option A of the readiness item proposes a report from inside the
    sandbox for failures that the guest cannot report. It names three: the
    start unit fails, the guest unit does not start, or the guest fails
    before it can report. Lines 43-44 put a failed decode of the file and a
    failed client build in that last group.
  - Its only source for the callback details is the registration file,
    "until the guest reads it".
  - `image/boat-run.sh:9-10` installs that file and removes the staged
    copy. So the file is the only copy of the details in the sandbox.
  - The guest opens, checks, and reads the file at `boat.rs:36-51`, and
    removes it at line 52. Only then does it decode the file, at lines
    53-54, and build its HTTPS client, at lines 55-60. On an error, the
    guest exits (`crates/ciproxy-guest/src/main.rs:15-17`). The guest unit
    has `Restart=no` (`image/ciproxy-boat-guest.service:11`).
  - With a usable registration file, A can cover these failures:
    - A failed start unit.
    - A guest unit that does not start.
    - A guest that finds the file but cannot open, check, or read it.
  - After the file removal, A cannot cover a failed decode or a failed
    HTTPS client build. As the item already says, A also misses a hang and
    a lost report.
  - Each of these two failures needs a corrupt file, a format mismatch
    between control plane and image, or a guest TLS setup failure. The
    backend writes the shared fields
    (`crates/ciproxy-overflow/src/service/launch.rs:22-27`), and the guest
    reads them (`boat.rs:20-25`). `crates/ciproxy-protocol/src` has no
    `deny_unknown_fields`, so an added field does not break an older guest.
  - The guest already reports a failed `runuser` spawn of the runner, as
    `RunnerExited` with no exit code, at
    `crates/ciproxy-guest/src/service.rs:70-73`. It also reports each
    runner exit (`service.rs:156-158`). A second report for the same
    token gets 401. So A must not add a report for these.
  - For a later coverage assessment, not an observed failure:
    `image/boat-run.sh` runs in the background
    (`crates/ciproxy-overflow/src/boat/launcher.rs:80`). If it fails before
    it queues the start unit, the start still becomes `Running`. Whether
    the registration file then exists depends on the line that failed.
    With no file, A has no callback details. The no-report list of the
    readiness item does not name this case yet.
- **Problem:** Option A says that it ends each guest failure before the
  guest can report. But the guest removes the file at `boat.rs:52`, before
  it decodes the file and builds its client. So A cannot report a failed
  decode or a failed client build.
- **Impact if no action:** The user can choose A and expect it to end these
  two failures at once. They still send no report, as today. Only a
  deadline, such as option B of the readiness item, or the sandbox lifetime
  ends such a start.
- **Root question:** How does the readiness item tell the user which
  unreported failures option A ends, and what A needs to end them?
- **Options:**
  - **A.** Limit the proposed report to failures that leave a usable
    registration file. Name the failures after the file removal that it
    cannot cover. Effort: Small. The steps:
    1. In option A of the readiness item, list the covered failures and the
       two uncovered ones above.
    2. Add the two uncovered failures to the sentence "A misses a hang and
       a lost report", and to the backstop of B in its Recommendation.
    3. Assess the background `boat-run.sh` failure, and put it in the right
       list.
    4. Keep the facts of today's reports: a failed runner start and each
       runner exit already report, and a second report gets 401.
  - **B.** Document a separate approach that keeps the callback details
    available after the file removal, and tie it to option A. Include the
    cost of protecting and deleting those details. Effort: Medium.
- **Left out:**
  - Change nothing, because the user decides the readiness item from this
    record.
  - Delete option A, because the narrowed A is better on every point. The
    known unreported cause, the disk measurement in the start unit, fails
    before the guest runs, so the narrowed A still covers it.
  - A control-plane check of the start and guest units through the Boat
    helper channel, which `launcher.rs:44-57` uses for its launch marker.
    It needs no callback details, but it is a new recovery option for the
    readiness item, not a correction of the coverage of A.
  - A guest change that removes the file after the decode and the client
    build, because it keeps a token longer. That retention is a choice for
    the user, and the text of A also stays wrong.
- **Wider fix:** None. The plans skill already requires an item that a
  reader can decide from.
- **Previous examples:** Commit `a3fd63be` corrected the same paragraph,
  after a review found a failed runner start in the wrong list. That edit
  dropped the text about the file that A needs, but it kept the claim.
- **Related:** `plans/follow-ups/boat-launched-runner-readiness.md` is the
  item to correct. If a branch closes it first, that branch deletes this
  item too.
- **Recommendation:** A. It makes the record true without a token retention
  change, which nobody has chosen.
- **Action:** fix after merge, apply A, because the branch that found it
  used both of its actual fix rounds, and the finding is not critical.
