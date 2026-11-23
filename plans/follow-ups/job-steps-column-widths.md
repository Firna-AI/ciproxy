# Step table columns are wider than the mockup

- **Severity:** Medium. **Likelihood:** Common. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-23, from the branch
  `calummoore/action-cpu-and-memory-usage`, review round 2, the two new
  problems of the check of finding 503.
- **Location:** `ts/app/src/screens/jobs/usage-steps.tsx:28-30` and `:36`,
  in `columns` and `killColumn`,
  `docs/mockups/src/screens/jobs/usage-steps.tsx:24-26` and `:32`, in the
  same two constants, and `docs/mockups/src/components/data-table.tsx:1-3`.
- **Reason:** Each option decides which side of a mismatch between the app
  and its rendered mockup is right, and only the user decides that.

- **Terms:**
  - **Steps table:** the table of the CPU and memory panel of the job page.
    It has a Step column with the number and the name of each step, then
    the figure columns.
  - **Figure columns:** the columns after Step: Duration, Average CPU, Peak
    memory, and Out of memory when a step ran out of memory.
  - **`DataTable`:** the table component of the mockups, in
    `docs/mockups/src/components/data-table.tsx`.
  - **Firna `Table`:** the `Table` component of the shared library
    `@firna/ui`. It has only fixed and flex columns.
- **Background:**
  - The app shows the steps table from 640 pixels, and a list below that
    (`ts/app/src/components/viewport.ts:12`). The mockups show the table in
    their desktop frame at 720 pixels.
  - The app gives the figure columns fixed widths of 88, 112, 112, and 128
    pixels, and gives the Step column the rest
    (`ts/app/src/screens/jobs/usage-steps.tsx:26-38`).
  - The mockup code sets the same widths
    (`docs/mockups/src/screens/jobs/usage-steps.tsx:22-34`). `DataTable`
    gives a fixed column its width in pixels, and gives the only flex
    column a preferred width of 100 percent.
  - The comment of `DataTable` says: "Fixed columns keep their width; flex
    columns share the rest as a preferred width"
    (`docs/mockups/src/components/data-table.tsx:1-3`).
  - With a flex column of 100 percent, Chrome makes each fixed column only
    as wide as its content, at every width. The rendered mockup columns
    are 73, 97, 101, and 108 pixels with padding at 960 pixels and below,
    and 81, 105, 109, and 116 pixels at 1,440 pixels.
  - The mockups are the spec of the app, by the design calls of this
    branch. A reader of a mockup sees the rendered table, not its code.
  - Before its merge, this branch gives plain and linked step names one
    name slot that can shrink, so a long name wraps inside the Step
    column.
  - Each figure uses non-breaking spaces. So a column must fit its longest
    value, such as "12 processes" or a vCPU figure with two digits.
  - Six mockup screens under `docs/mockups/src/screens/` give fixed widths
    to their table columns: the job tables, the steps table, the machine
    table, the label list, the members, and the organization list.
  - `plans/follow-ups/firna-ui-table-roles.md` plans to move the steps
    table back to the Firna `Table`.
- **Problem:**
  - At 640 and 720 pixels, the Step column of the app is 85 pixels
    narrower than that of the mockup: 66 against 151 pixels, and 146
    against 231 pixels. At 640 pixels, a name in the app then takes four to
    seven lines.
  - The mockup code and its rendered table disagree. The code and the
    comment of `DataTable` say 88, 112, 112, and 128 pixels, but Chrome
    draws the content width.
- **Impact if no action:** A long step name takes up to seven lines at 640
  pixels, so the table gets tall and hard to scan. The app and the mockup
  keep two layouts, and the comment of `DataTable` misleads the next
  change of a mockup table.
- **Root question:** Which widths should the figure columns have, so that
  the Step column keeps room for the names, and the app and the mockup
  show the same table?
- **Options:**
  - **A.** Keep the app widths, and make the mockup render them. Make
    `DataTable` keep the width of each fixed column, for example with a
    minimum width. Cost: the Step column of the mockup gets as narrow as
    that of the app, and the other five mockup screens with fixed columns
    can change their look too. Effort: Medium.
  - **B.** Give both sides the widths of the rendered mockup, as fixed
    widths.
    - Set the four figure columns to fixed widths that fit their longest
      values, such as 64, 84, 88, and 100 pixels, in the app and in the
      mockup code. Correct the comment of `DataTable`.
    - In Chrome, with the one name slot and these widths, the Step column
      got about 150 to 170 pixels at 640 pixels. Every name stayed inside
      its cell, in one to three lines.
    - Fixed widths stay valid after the move to the Firna `Table`.
    - Cost: each width must fit the longest value. A longer new figure,
      such as a vCPU value with three digits, needs a new width.
    - Effort: Medium.
  - **C.** Size each figure column to its content in the app, as the
    mockup renders it, and correct the comment of `DataTable`. Cost: the
    Firna `Table` has only fixed and flex columns, so C conflicts with the
    move of `plans/follow-ups/firna-ui-table-roles.md`. The app table also
    needs a measure of its own. Effort: Medium.
  - **D.** Do B, and also make `DataTable` keep the width of each fixed
    column, so that every mockup table renders the widths that its code
    gives. Cost: the other five mockup screens with fixed columns can
    change their look. Each then needs the same check against its app
    table. Effort: Large.
- **Left out:**
  - Cut names, because they hide the text that tells
    `actions/checkout@v4` from `actions/setup-node@v4`, and the mockup
    wraps names.
  - The phone list at these widths, because the mockups show the table in
    the desktop frame at 720 pixels.
  - No Out of memory column on a narrow window, for the same reason.
  - A corrected comment with no change of the layout, because the app and
    the mockup then keep two tables, so it fails the goal.
- **Wider fix:** D. The mockup then renders the widths that its code gives
  for every table, so a width that the app copies from the mockup code
  cannot drift from the approved look.
- **Previous examples:** None found.
- **Related:** `plans/follow-ups/firna-ui-table-roles.md`, the same code.
  Its option A moves the steps table back to the Firna `Table`, which has
  only fixed and flex columns. A, B, and D keep fixed and flex columns, so
  they work with that move. C conflicts with it.
- **Recommendation:** B. It gives the app the table that the rendered
  mockup shows, and fixed widths work with the Firna `Table`. D best
  protects the codebase, because each mockup table then renders the widths
  that its code gives.
- **Action:** ask the user, because each option decides which side of a
  mismatch between the app and the rendered mockup is right.
