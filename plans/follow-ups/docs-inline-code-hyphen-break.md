# Docs inline code breaks at a hyphen

- **Severity:** Low. **Effort:** Small. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-23, from the branch `calummoore/diagram-request-3jpv`,
  found in the screenshots of review fix round 1.
- **Source:** Implementation.
- **Category:** Mockup.
- **Location:** `ts/site/src/styles/starlight/content.css:59` and
  `docs/mockups/src/styles/base.css:191`.
- **Reason:** The rule covers every docs page, and the branch that found it
  changes only the page How ciproxy works.
- **Terms:**
  - **Inline code:** a code span in the text of a docs page, such as
    `ubuntu-latest`.
- **Background:**
  - The mockup class `.inline-code` keeps a code span on one line.
  - The site rule for inline code in docs pages, at
    `ts/site/src/styles/starlight/content.css:59`, does not.
  - At a 1440 px window, the site page How ciproxy works breaks
    `ubuntu-latest` into "ubuntu-" and "latest" at the end of a line. The
    mockup keeps it whole.
- **Problem:** The site and its mockup differ, and a reader can copy half of
  a label name.
- **Impact if no action:** Some label and image names show in two parts on
  the docs pages.
- **Root question:** How should docs inline code wrap on a narrow line?
- **Options:**
  - **A.** Add `white-space: nowrap` to the site rule, as the mockup has it.
    Check that no long code span overflows a 390 px phone. Effort: Small.
  - **B.** Keep the site rule, and let the mockup wrap code spans too. A
    name can then still break at a hyphen. Effort: Small.
- **Left out:** A rule for each code span, because the class applies to
  all docs pages.
- **Wider fix:** None. The `mockups` skill already requires the site and the
  mockups to agree.
- **Previous examples:** None found.
- **Recommendation:** A. It follows the mockup, which is the spec.
- **Action:** fix after merge, apply A, because the rule covers every docs
  page.
