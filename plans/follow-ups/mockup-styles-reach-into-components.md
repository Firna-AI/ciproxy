# Jobs mockup styles reach into shared components

- **Severity:** Low. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-23, from the branch
  `calummoore/action-cpu-and-memory-usage`, review round 1: a problem that
  the check of finding 206 found outside the scope.
- **Location:** `docs/mockups/src/screens/jobs/jobs.css:74` and `:125`, and
  `docs/mockups/src/components/data-table.tsx:22`.
- **Reason:** The two rules are on `main`, outside the change that found
  them.

- **Terms:**
  - **DataTable:** the shared table component of the mockups.
  - **Dot:** the shared status dot component of the mockups.
- **Background:**
  - The `typescript` skill says that a component gives props, and other
    code does not change its elements through their class names.
  - The branch that added the CPU and memory panel fixed the same coupling
    in `docs/mockups/src/screens/jobs/job-usage.css`: the screen now
    styles content that it owns.
- **Problem:**
  - `jobs.css:74`: `.job-row > .dot` sets the offset of the shared `Dot`
    through its class.
  - `jobs.css:125`: `.mockup-desktop .dt .col-finished` hides the
    `finished` column through the `.dt` and `col-<key>` classes of
    `DataTable`. `DataTable` emits `col-<key>` only for screen rules such
    as this one (`docs/mockups/src/components/data-table.tsx:22`), and
    `data-table.css` never reads it.
- **Impact if no action:** A change to the markup of `Dot` or `DataTable`
  can undo the Jobs layout without an error.
- **Root question:** How can the Jobs mockup set the dot offset and hide a
  column without the markup of another component?
- **Options:**
  - **A.** Let the job row own the dot offset with an element that the row
    renders, and give `DataTable` a column option for the hide on a narrow
    window. Then stop emitting `col-<key>`. Effort: Small.
  - **B.** Scope the classes of `DataTable` and `Dot` with the
    `.module.css` support of Mokly, so that no screen stylesheet can select
    them. No other part of the repository uses CSS Modules, and a screen
    can still select an element such as `td`. Effort: Medium.
- **Left out:** A policy check that fails when a screen selects the class
  of another component, because it needs the user's approval as a new
  gate, and it misses element selectors.
- **Wider fix:** None, because no option stops a screen from selecting an
  element of another component.
- **Previous examples:** The fix in `job-usage.css` above.
- **Recommendation:** A. It removes both couplings with the smallest
  change.
- **Action:** fix after merge, apply A, because the rules are on `main`
  and outside the change that found them.
