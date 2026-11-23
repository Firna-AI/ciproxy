# Firna tables give no table roles

- **Severity:** Medium. **Likelihood:** Common. **Effort:** Large.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-23, from the branch
  `calummoore/action-cpu-and-memory-usage`, review round 1: the wider fix
  of finding 203, and a problem that its check found outside the scope.
- **Location:** `ts/app/src/screens/jobs/job-list.tsx:112`, `:166`, and
  `:202`, `ts/app/src/screens/savings/savings-panels.tsx:99`,
  `ts/app/src/screens/settings/label-list.tsx:82`,
  `ts/app/src/screens/settings/member-rows.tsx:152`, and
  `ts/app/src/screens/settings/organization-list.tsx:138` and `:161`.
- **Reason:** The fix lives in the `@firna/ui` repository and needs a
  library release.

- **Terms:**
  - **Firna Table:** the `Table` component of the shared library
    `@firna/ui`, which the dashboard uses for its wide tables.
  - **Table roles:** the table, row, column header, and cell roles that
    let a screen reader move by row and by column and name the header of
    each cell.
- **Background:**
  - Firna Table draws its columns with plain views. It gives no table
    roles. Chrome finds none of these roles on a Firna table at 1,440
    pixels.
  - The Machines table gives the roles itself
    (`ts/app/src/screens/machines/machine-table.tsx:145`). The branch that
    added the CPU and memory panel gave the steps table of the job page the
    roles in the app, with the Firna look from `createTableStyles` of
    `@firna/ui/table`.
  - Eight Firna tables on `main` give no roles: the three tables of
    `job-list.tsx`, the table of `savings-panels.tsx`, the label list, the
    member rows, and the two tables of `organization-list.tsx`, at the
    lines of **Location**.
- **Problem:** A screen reader cannot move through these tables by row or
  by column, or ask which header belongs to a value.
- **Impact if no action:** Each new table that uses Firna Table repeats
  the gap, and each fix in the app adds one more copy of the roles.
- **Root question:** How can every dashboard table give its rows, columns,
  and headers to assistive technology?
- **Options:**
  - **A.** Add the table roles to Firna Table in the `@firna/ui`
    repository, and take the release. Then remove the app's own roles from
    the steps table and the Machines table where the Firna roles cover
    them. Effort: Large.
  - **B.** Add an app table control in `ts/app/src/components` with the
    Firna look (`createTableStyles` and `useSharedUiTheme`) and the roles,
    and move the eight tables to it. It is a second table control beside
    Firna Table, and A makes it redundant. Effort: Medium.
  - **C.** Add the roles in each of the eight tables, as the Machines
    table does. Effort: Medium.
- **Left out:** A label on each table wrapper, because it gives no
  headers or cells. The Firna `DataGrid`, because its cells hold only
  typed values, so it cannot hold links or status labels.
- **Wider fix:** A. It fixes the shared table, so each new table gets the
  roles.
- **Previous examples:** `plans/follow-ups/firna-ui-label-weight-token.md`
  waits on a Firna change for the same reason.
- **Recommendation:** A. It fixes every consumer at the source. B is the
  fallback if the library change cannot land.
- **Action:** fix after merge, apply A, because the fix lives in the
  `@firna/ui` repository and needs a library release.
