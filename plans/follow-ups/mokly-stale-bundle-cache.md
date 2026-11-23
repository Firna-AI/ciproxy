# Mokly build fails after a CSS-only change

- **Severity:** Medium. **Effort:** Medium. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-23, from the branch `calummoore/diagram-request-3jpv`,
  during the mockup milestone of the How ciproxy works diagram.
- **Source:** Implementation.
- **Category:** Process.
- **Location:** `ts/mockups/package.json`, the `build` script, and the Mokly
  cache folder `~/.cache/mokly/<hash>/bundles` on the machine that builds.
- **Reason:** The branch that found it changes only mockup sources and docs,
  not the mockup tooling.
- **Terms:**
  - **Mokly:** the tool that builds the mockup catalogue under
    `docs/mockups/` into `docs/mockups/mokly-generated/`. The repository pins
    `@mokly/mokly` 0.21.0.
- **Background:**
  - `npm --prefix ts run mockups:check` runs
    `mokly build --strict --no-changes --out docs/mockups/mokly-generated`
    first.
  - After a change to only a CSS file of a screen, the build failed four
    times with this text:
    `[mokly/export-invalid] Inputs changed during the build: …/job-path-figure.css. Build again.`
  - No other process changed the file during the build.
  - Deleting `~/.cache/mokly/<hash>/bundles` made the next build pass. A
    second CSS-only change made the failure come back.
  - A CI run starts with no Mokly cache, so CI did not meet it.
- **Problem:** A local mockup check fails after a CSS-only change until the
  person deletes the Mokly bundle cache by hand.
- **Impact if no action:** Each person and agent who changes mockup CSS loses
  time, and can think that the change itself is wrong.
- **Root question:** How does a local mockup build stay correct after a
  CSS-only change?
- **Options:**
  - **A.** Add the cache step to the visual smoke test of
    `docs/mockups/README.md`: on this error, delete the bundle cache and
    build again. It hides the cause. Effort: Small.
  - **B.** Make a small reproduction and report it to Mokly. Pin the fixed
    version when it ships. It waits for the Mokly maintainers. Effort:
    Medium.
  - **C.** Make the `build` script clear the bundle cache before each build.
    Each build is then slower. Effort: Small.
- **Left out:** Leave out `--no-changes` from the build. That flag guards the
  catalogue against a change during the build, so this weakens a check.
- **Wider fix:** B, because it removes the cause for every user of Mokly.
- **Previous examples:** None found.
- **Recommendation:** B, with A in the meantime. A note costs little, and the
  cause belongs to Mokly.
- **Action:** fix after merge, apply B, because the fix belongs to the mockup
  tooling and not to the docs change that found it.
