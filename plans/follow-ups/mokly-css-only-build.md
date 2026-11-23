# Mokly build fails after a CSS-only edit

- **Severity:** Low. **Effort:** Small. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-23, from the branch
  `calummoore/action-cpu-and-memory-usage`, a problem that the mockup work
  of its plan found.
- **Location:** `ts/mockups/package.json:6` and `docs/mockups/README.md`.
- **Reason:** The problem is in the mockup tooling, not in the change that
  found it.

- **Terms:**
  - **Mokly:** the tool that builds the mockup catalogue from the entry
    modules under `docs/mockups/src/`. The repository pins
    `@mokly/mokly` 0.21.0 in `ts/mockups/package.json`.
  - **Build cache:** the folder `~/.cache/mokly`, where Mokly keeps digests
    of its inputs between builds.
- **Background:**
  - `npm --prefix ts run mockups:check` first runs
    `mokly build --strict --no-changes`.
  - During the mockup work of 23 November 2026, an edit to one CSS file
    under `docs/mockups/src/screens/jobs/` made every later build fail
    with `[mokly/export-invalid] Inputs changed during the build:
    …/screens/jobs/job-usage.css. Build again.`
  - The build cache kept the old digest of the CSS file. An edit to a
    TypeScript file cleared the problem.
  - Nobody reproduced it in a clean checkout.
- **Problem:** After an edit to a CSS file only, the strict mockup build
  can fail on every run until a TypeScript file changes.
- **Impact if no action:** An agent or a person who edits only mockup CSS
  sees a failed check with no hint at the cause.
- **Root question:** How does a CSS-only edit to the mockups build at once?
- **Options:**
  - **A.** Add the workaround to the "Commands" section of
    `docs/mockups/README.md`: delete `~/.cache/mokly`, or touch a
    TypeScript file, and build again. Effort: Small.
  - **B.** Make the `build` script in `ts/mockups/package.json` delete
    the Mokly build cache first. Cost: each build is slower. Effort: Small.
  - **C.** Report the bug to the Mokly project, and update the pin when a
    fix ships. Cost: the wait depends on that project. Effort: Small.
- **Left out:** None.
- **Wider fix:** C, because a fix in Mokly removes the case for every
  repository that uses it.
- **Previous examples:** None found.
- **Recommendation:** C. A fix in Mokly removes the cause for everyone. A
  helps readers of this repository until then.
- **Action:** ask the user, because C sends a report to an outside project.
