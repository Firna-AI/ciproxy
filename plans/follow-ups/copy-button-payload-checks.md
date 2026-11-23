# Copy button checks do not read the clipboard

- **Severity:** Medium. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-23, from the branch `calummoore/copying-llm-xioi`,
  review round 2, a problem outside the scope that the check of finding 10
  raised.
- **Location:** `ts/app/scripts/browser-actions.ts`, the `setup-waiting` and
  `setup-available` branches of the shell smoke.
- **Reason:** The gap is older than the branch that found it.

- **Terms:**
  - **Shell smoke:** `ts/app/scripts/browser-smoke.ts`. It drives headless
    Chrome against the fixture harness `ts/app/scripts/harness-server.ts`.
- **Background:** Setup has three copy buttons: "Copy the install command",
  "Copy the workflow change", and "Copy prompt for your agent". The shell
  smoke grants clipboard access and presses each one. Only the check of the
  prompt button reads the clipboard and compares the text, with
  `page.evaluate(() => navigator.clipboard.readText(), undefined)`.
- **Problem:** The checks of the install command and the workflow change
  only check the "Copied" button name. A change that copies the wrong text,
  such as `useCopy("")`, passes them.
- **Impact if no action:** The install command or the workflow change can
  copy the wrong text while every check passes.
- **Root question:** How do the browser checks prove that each copy button
  copies its text?
- **Options:**
  - **A.** Read the clipboard after each press and compare it with the text
    that the page shows or that the fixture gives. Effort: Small.
  - **B.** Add one shared helper that presses a copy button, waits for
    "Copied", and returns the clipboard text. Use it for all three
    buttons. Effort: Small.
- **Left out:** A unit test of the formatter cannot catch the wiring of a
  button.
- **Wider fix:** B.
- **Previous examples:** The prompt button check in
  `ts/app/scripts/browser-agent-prompt.ts`.
- **Recommendation:** B. Every copy button then gets the same check.
- **Action:** fix after merge, apply B, because the checks are older than
  the branch that found the gap.
