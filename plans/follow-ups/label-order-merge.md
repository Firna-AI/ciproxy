# An old label dialog can erase an order change

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-23, from the branch
  `calummoore/ciproxy-providers-interface-s8rp`, review finding 3, the
  wider fix E.
- **Location:** `crates/ciproxy-store-pg/src/labels.rs`,
  `crates/ciproxy-api/src/handlers/labels.rs`, and
  `ts/app/src/screens/settings/label-dialog.tsx`.
- **Reason:** The merge rule is new behaviour, and the user reviews each
  new requirement.

- **Source:** Review.
- **Category:** Product bug.
- **Terms:**
  - **Label order:** the list of places, own machines and each provider,
    where the jobs of one label run.
  - **Offered provider:** a provider that the workspace has connected at
    least once. Its first connection adds it to the end of each Linux
    label's order.
- **Background:**
  - The label dialog edits the size, the "When nothing fits" choice, and
    the order of one label.
  - The fix of review finding 3 makes the dialog send the order only when
    the person changed it. The server keeps the stored order otherwise. So
    a size-only save no longer erases a provider that a first connection
    added while the dialog was open.
  - An order edit still sends the whole order that the dialog shows.
- **Problem:** When a person edits the order in a dialog that opened before
  another change, such as a first connection or a second Admin's edit, the
  save replaces the current order. A provider that the editor never saw
  leaves the label, and a later connection does not add it back.
- **Impact if no action:** In this rare case, jobs of the label skip a
  provider that the customer connected, until someone adds it to the label
  again.
- **Root question:** How does an order edit keep the changes that its
  editor did not see?
- **Options:**
  - **A.** Require a matching revision for an order edit, and ask the
    editor to reload when it changed. This adds a refusal. It needs a
    revision column. Effort: Medium.
  - **B.** Send the order that the dialog opened with beside each order
    edit, and refuse the edit when the current order differs. This adds a
    refusal. Effort: Medium.
  - **C.** Record each move, addition, and removal in the dashboard, and
    apply them to the current order. Effort: Large.
  - **E.** Send the opened order beside each order edit. Inside the update
    transaction, apply the places that the editor added, removed, or moved
    to the current order. Keep each place that the editor never saw, and
    put a newly offered provider at the end. When two people move the same
    place, the later move wins. State the rule in
    `docs/protocol/labels-order.md`. Effort: Medium.
- **Left out:** A fresh read when the tab becomes visible again, because two
  windows side by side and a second Admin still save old data. Adding
  every offered provider back at each save, because it breaks the rule that
  a removal stays.
- **Wider fix:** E. It removes the wrong case for every save with no
  refusal.
- **Previous examples:** `plans/follow-ups/overflow-policy-stale-writes.md`
  records the same lost update for the overflow settings.
- **Related:** `plans/follow-ups/control-plane-schema-rollout.md` only for
  option A, which adds a column.
- **Recommendation:** E. It keeps a recoverable edit instead of refusing it,
  as the rule "prefer graceful handling" asks.
- **Action:** ask the user, because E adds a merge rule that no requirement
  covers, and A and B add a refusal.
