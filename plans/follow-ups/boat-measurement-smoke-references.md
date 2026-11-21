# Measurement item cites the wrong smoke lines

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-21, from `conductor/boat-locale-fix`, final review
  round 4, finding 11.
- **Location:** `plans/follow-ups/boat-live-root-measurement-race.md:47-49`.
  The wrong citation is on line 48.
- **Reason:** The branch that found it had used both of its actual fix
  rounds, and no third fix round was authorized.

- **Terms:**
  - **Measurement item:** the follow-up
    `plans/follow-ups/boat-live-root-measurement-race.md`. It records a
    Boat start that fails when Boat deletes a file during a `du`
    measurement of the whole disk.
  - **Smoke guide:** `image/boat-smoke.md`, the operator guide for live
    Boat checks.
- **Background:**
  - To support its options D and F, lines 47-49 of the measurement item
    say that operators read only the free bytes in the start journal.
  - That sentence cites `image/boat-smoke.md:184-185` and `:208`,
    `image/README.md:165-166`, and `image/boat.md:175`. The last two
    citations are correct.
  - The two smoke guide statements that support the claim are under its
    heading [GitHub job smoke](../../image/boat-smoke.md#github-job-smoke):
    - "The journal of `ciproxy-boat-start.service` shows the free bytes
      before the guest starts."
    - "Keep the full software set on `large` and measure its actual free
      job disk."
  - The citations were correct when the item was added. A later fix of
    the locale probe added 4 lines in the restore section of the smoke
    guide, and it did not update the item.
  - The Markdown gate fails on a link to a missing file, and it warns on a
    link to a missing heading (`xtask/README.md:46-49`). It does not check
    a `path:line` citation in text.
  - If a branch deletes the measurement item first, for example when it
    applies that item's option F, nothing is left to fix. That branch then
    deletes this item too.
- **Problem:** The cited smoke guide lines now show the end of a GitHub job
  lookup command and a link to the overflow guide. They do not show the two
  statements.
- **Impact if no action:** A reader must search the smoke guide to check
  the claim. The claim keeps its support from the two correct citations in
  the same sentence.
- **Root question:** How does the measurement item cite the two smoke guide
  statements about the free bytes, so that later guide edits keep the
  citation correct?
- **Options:**
  - **A.** Correct the line numbers. A later edit of the guide can move
    them again. Effort: Small.
  - **B.** Link to the heading, with the target
    `../../image/boat-smoke.md#github-job-smoke`, and name the two
    statements by their text. The Markdown gate checks the link. Effort:
    Small.
  - **C.** Cite only the Boat launch guide, `image/boat.md`. It describes
    the journal output, but it leaves out the smoke procedure. Effort:
    Small.
  - **D.** Add a sub-heading such as `### Free job disk` to the smoke
    guide. Put the two statements under it, and link to that heading. It
    reorders an operator guide only to serve a citation in a plan item.
    The two statements are 24 lines apart, with the delete helper text
    between them. A later rename of the heading gives only a gate warning.
    Effort: Small.
- **Left out:**
  - A gate check of line citations in text, because a `path:line` citation
    holds no expected text. Such a check can only confirm that the line
    exists, and the wrong lines exist. It is also a new gate, which the
    user must approve.
  - A rule that each change updates the citations of the lines that it
    moves, because the documentation skill already covers docs drift.
  - A relative line link, such as `boat-smoke.md?plain=1#L188-L189`,
    because it goes out of date like A, and the gate checks only its file
    part.
  - A permalink at a fixed commit, because it shows an old copy of the
    guide, not the live guide.
  - The statement text with a plain file path and no link, because B names
    the same statements, and the gate also checks its link.
- **Wider fix:** None. The documentation skill already gives the rule: each
  fact lives in one place, and other docs link to it.
- **Previous examples:** Commit `3428fd37` includes the part "docs(plans):
  point follow-up items to the moved lines". It gave follow-up items the
  new line numbers of the files that its change edited. That was option A,
  in the same change that moved the lines.
- **Recommendation:** B. A heading link with the text of both statements
  stays correct after each guide edit that keeps `## GitHub job smoke`,
  and the gate checks it. The line numbers of A can move again before a
  later branch applies the fix.
- **Action:** fix after merge, apply B, because the branch that found it
  used both of its actual fix rounds, and no third round was authorized.
