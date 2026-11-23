# Follow-up items cite lines that moved

- **Severity:** Low. **Effort:** Small. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-23, from the branch `calummoore/diagram-request-3jpv`,
  the wider fix of review finding 9 and two problems outside its scope.
- **Source:** Review.
- **Category:** Process.
- **Location:** `plans/follow-ups/current-addon-payment-scenes.md:47`,
  `plans/follow-ups/boat-image-services.md`, and the `plans` skill of
  `futex-ai/skills`, section "Keeping an item current".
- **Reason:** The skill lives outside this repository, and the two stale
  items are outside the diff of the branch that found them.
- **Terms:**
  - **Follow-up item:** a file under `plans/follow-ups/` that records work
    for after a merge. It names live files by path, often with a line.
- **Background:**
  - The branch above edited the page How ciproxy works. The follow-up item
    `boat-image-services.md` cited lines of that page, which then pointed
    to a blank line and to old words. The branch changed those references
    to Markdown links to the heading, which the Markdown phase of the gate
    checks.
  - The `plans` skill asks an item to stay current when a change replaces a
    decision or edits a protocol section. It does not ask this when a change
    edits another file that an item cites.
  - `current-addon-payment-scenes.md:47` cites
    `crates/ciproxy-api/src/handlers/billing_views.rs:109`, but that file
    has 84 lines.
  - `boat-image-services.md` still asks the user, although the user chose
    its option E on 19 November 2026, as `plans/ciproxy-mvp.md` records.
    Its references `image/boat-run.sh:20` and `:26`, `image/boat.md:9`, and
    `docs/protocol/agent-images.md:5-8` and `:24-26` no longer point to the
    cited text. `plans/ciproxy-mvp.md` already has the TODO that deletes the
    item.
- **Problem:** A change can move the lines that an item cites, and no rule
  or check makes the change update the item.
- **Impact if no action:** A later agent follows an item to the wrong line
  and can edit the wrong text.
- **Root question:** How does a follow-up item keep pointing to the text
  that its work needs?
- **Options:**
  - **A.** Fix the stale reference of `current-addon-payment-scenes.md`.
    Leave `boat-image-services.md` to the TODO that deletes it. Effort:
    Small.
  - **B.** Do A. Add a rule under "Keeping an item current" in the `plans`
    skill: a change that edits a file also searches `plans/follow-ups/` for
    its path and updates each cited line. Prefer a Markdown link to a
    heading, which the Markdown phase checks. Effort: Small.
- **Left out:** A gate check of backticked `path:line` text, because a new
  check needs the user and a heading link already gets the existing check.
- **Wider fix:** B, because the rule covers every edit of a cited file.
- **Previous examples:** Review finding 9 of the branch above.
- **Recommendation:** B. The skill change prevents the next stale line.
- **Action:** fix after merge, apply B, because the skill lives outside this
  repository.
