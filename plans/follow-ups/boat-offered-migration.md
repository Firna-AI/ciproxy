# Migration marks Boat offered in every workspace

- **Severity:** Medium. **Effort:** Small. **Confidence:** High.
  **Harder later:** Yes.
- **Kind:** Needs decision.
- **Added:** 2026-11-24, from the branch
  `calummoore/ciproxy-providers-interface-s8rp`, review finding 13.
- **Location:**
  `crates/ciproxy-schema/migrations/202611231601_overflow_provider_settings_and_label_order/up.sql:15`,
  `docs/protocol/overflow.md:21` and `:69`, and
  `ts/app/src/screens/overflow/provider-panel.tsx:101`.
- **Reason:** A and D trade old label choices against the documented
  first-connection rule, and only the user chooses.

- **Source:** Review.
- **Category:** Docs or spec.
- **Terms:**
  - **Offered provider:** a provider that the workspace has connected at
    least once. Its first connection appends it to the end of each Linux
    label's order, and new Linux labels get it.
- **Background:**
  - The migration gives each existing label an order from its old switches,
    "Your machines" and "Boat", so routing does not change.
  - It also marks Boat as offered in every existing workspace, including
    workspaces that never connected Boat.
  - So in such a workspace, a Linux label that had Boat switched off keeps
    Boat out after the first Boat connection. The protocol, the site, and
    the Settings caption describe the plain first-connection rule.
  - Old data can still show a past connection: each connection on `main`
    saved a Boat spend limit, which stays after Disconnect and after the key
    rows are deleted.
- **Problem:** The code and the documented first-connection rule disagree
  for workspaces that existed before the migration and never connected
  Boat.
- **Impact if no action:** An Admin who connects Boat for the first time can
  expect every Linux label to use it, but labels that had Boat off skip it.
  The Settings caption also tells such a workspace to "connect it again".
- **Root question:** When does Boat count as offered in a workspace that
  existed before the label order?
- **Options:**
  - **A.** Keep the migration as it is, and state the exception in the
    protocol, the site, the Settings README, and the caption. Effort: Small.
  - **B.** Keep each migrated label's old Boat choice apart from the offered
    mark. Effort: Medium.
  - **C.** Apply the first-connection append to workspaces that never
    connected Boat. Effort: Large.
  - **D.** In the migration, mark Boat as offered only where old data shows a
    connection: a saved Boat spend limit or a Boat row in
    `overflow_provider_accounts`. Other workspaces follow the documented
    rule. Run the migration once on a connected, a disconnected, and a
    never-connected workspace, and save the result under `.context/`.
    Update the design calls of the plan. Effort: Small.
- **Left out:** An active key as the proof, because a workspace can connect
  and later disconnect Boat.
- **Wider fix:** None. The `documentation` skill already requires the code
  and the protocol to agree.
- **Previous examples:** None found.
- **Related:** `plans/follow-ups/label-order-merge.md` uses the term
  "offered", which A would also need to qualify.
- **Recommendation:** D. The code then follows the user's decision of 23
  November 2026 and the current docs, with no lasting exception. Its one
  cost is a one-time change, announced in the panel, to Boat-off labels in
  workspaces that never connected Boat, and such workspaces exist only
  before production.
- **Action:** ask the user, because A keeps old label choices, and D changes
  them once to follow the documented rule.
