# Diagram boxes and job list are two copies

- **Severity:** Low. **Effort:** Medium. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-23, from the branch `calummoore/diagram-request-3jpv`,
  the wider fix of review finding 2.
- **Source:** Review.
- **Category:** Code structure.
- **Location:** `ts/site/src/data/job-path.ts`,
  `ts/site/src/content/docs/docs/how-ciproxy-works.mdx` under "A job on
  your machines", `docs/mockups/src/screens/docs/job-path-figure.tsx`, and
  `docs/mockups/src/screens/docs/how-ciproxy-works.tsx`.
- **Reason:** A shared source changes how the page and the mockup render the
  numbered list, which the branch that added the diagram did not need.
- **Terms:**
  - **Job path figure:** the diagram on the docs page How ciproxy works. Its
    boxes 01 to 04 are the steps of the list "A job on your machines".
- **Background:**
  - The figure text lives in `ts/site/src/data/job-path.ts` on the site, and
    in `job-path-figure.tsx` in the mockups.
  - The numbered list is Markdown on the site page, and JSX in the mockup
    screen.
  - Review finding 2 found that box 03 and list step 3 named different
    events. The branch fixed the texts by hand.
- **Problem:** The boxes and the list are written twice in each package, so
  a later edit of one can give a number two meanings again.
- **Impact if no action:** A reader who follows a box number to the list can
  read a different event.
- **Root question:** How do the boxes and the list keep one step for each
  number?
- **Options:**
  - **A.** Keep the two copies, and add a comment beside each that names the
    other. Effort: Small.
  - **B.** Render the list and the boxes from one ordered set of steps, with
    the part, the short box text, and the full list text of each. The site
    page shows the list through a `jsx()` call. The set needs a form for
    inline code. Effort: Medium.
- **Left out:** A test that compares the texts, because the `testing` skill
  says that doc prose gets no test.
- **Wider fix:** B, because one set removes the second copy.
- **Previous examples:** Review finding 2 of the branch
  `calummoore/diagram-request-3jpv`.
- **Recommendation:** B. It removes the cause in each package.
- **Action:** fix after merge, apply B, because it changes how the page
  renders its list.
