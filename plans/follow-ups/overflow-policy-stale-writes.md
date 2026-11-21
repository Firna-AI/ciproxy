# Overflow settings saves send the whole policy

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-08, from Milestone 22, pull request #51, its handoff.
  On 2026-11-17, the final review of the branch
  `calummoore/ci-proxy-load-balancer` found the same cause as finding 1004:
  a save of the switch or the wait needs a spend limit. Its check showed
  that one request change fixes both problems, so finding 1004 joined this
  item.
- **Source:** Implementation and Review.
- **Category:** Product bug.
- **Location:** the Overflow settings route of `crates/ciproxy-api`: `save`
  in `crates/ciproxy-api/src/handlers/overflow.rs`, `OverflowPolicy` in
  `crates/ciproxy-api/src/wire/overflow.rs`, and
  `crates/ciproxy-api/overflow.md`. The Boat settings of `ts/app`:
  `policyFrom` in `ts/app/src/state/overflow.ts`, and the panels under
  `ts/app/src/screens/overflow/`. Also `save_settings` in
  `crates/ciproxy-store-pg/src/settings.rs`.
- **Reason:** The fix needs backend work outside the Milestone 22 scope, and
  each defect needs two editors at once or a direct API call.

- **Terms:**
  - **Overflow policy:** the overflow switch, the wait, and the spend limit
    of a workspace. Settings under Overflow edits them.
  - **Spend limit:** the monthly bound of the customer on Boat spending, an
    amount above zero.
  - **Policy request:** `PUT /workspaces/{workspace_id}/overflow` with the
    body `OverflowPolicy`: `enabled`, `wait_seconds`, and
    `spend_limit_cents`.
- **Background:**
  - The Overflow settings save the whole policy: the wait, the spend limit,
    and the Boat state. Two tabs or two Admins can edit them.
  - `crates/ciproxy-api/overflow.md` says that the policy request saves the
    complete policy, with a `spend_limit_cents` above zero.
  - The handler `save` turns the limit with `overflow_views::limit`, which
    refuses zero or less. So every save needs a positive limit, also a save
    with `enabled: false`.
  - The store already accepts a missing limit when none is stored.
    `save_settings` refuses a missing limit only when a limit is stored.
    Only the API handler adds the requirement.
  - A workspace that skipped Boat has no spend limit. Its settings still
    hold the switch and the wait.
  - The dashboard shows the switch and the wait only when a Boat account
    and a limit exist, in `overflow-screen.tsx`. The mockup state with no
    connection does not show them either. `policyFrom` sends no save
    without a limit.
  - So only a direct API call meets the spend-limit tie. The live pass of
    the docs on 17 November 2026 met it, and it saved an invented limit of
    one cent.
  - The API cannot clear such a limit. The next Connect form shows the
    saved limit as its first value, in `boat-panel.tsx`.
  - A connection changes only the limit and keeps the saved switch, in
    `crates/ciproxy-store-pg/src/overflow/accounts.rs`. So an off state
    saved before Boat connects carries into the next connection.
  - "Spend limit" in `docs/protocol/overflow.md` asks for the limit at each
    connection. The limit stays above zero, and the customer can change it
    at any time. The section rejects a default limit. It sets no request
    shape.
  - The dashboard already edits the limit in its own panel,
    `spend-panel.tsx`.
  - The Boat reservation check in
    `crates/ciproxy-store-pg/src/overflow/admission.rs` refuses a start with
    a spend reading while no limit is stored. Each connection saves a limit.
  - Before Boat connects, the switch changes no start, as
    `docs/protocol/job-routing.md` says.
- **Problem:** Each save sends the complete policy. This gives two defects:
  - A save from an older page writes back the values that it read, also
    those that another Admin changed since.
  - Every save needs a positive spend limit, also a save of the switch or
    the wait alone. A workspace that skipped Boat cannot save the off state
    without inventing a spending amount.
- **Impact if no action:** A newer spend limit or wait can be lost silently.
  A direct API caller must invent a limit to save the switch or the wait,
  and the API cannot clear that limit later.
- **Root question:** How does a save of the overflow policy change only what
  its person changed, with no need to send the other fields?
- **Options:**
  - **A.** Allow an omitted limit when the switch is off. Keep an existing
    limit when the request omits it. Keep a positive limit mandatory for a
    connection and for a policy with the switch on. Cost: a save of the
    wait with the switch on still needs a limit. D is better than A on
    every point. Effort: Small.
  - **B.** Save only the changed fields. A save of the switch or the wait
    keeps the stored limit. Change the API, the client, and the regression
    tests together. Cost: it changes the documented complete-policy
    request. Two editors of the same field still overwrite each other.
    Effort: Medium.
  - **C.** Add a separate operation for the switch, and keep the complete
    operation for the other fields. Cost: a save of the wait still needs a
    limit. Effort: Medium.
  - **D.** Make `spend_limit_cents` optional in every save, with the switch
    on or off.
    - A missing limit keeps the stored limit, and a limit in the request
      stays above zero. The dashboard sends the limit only from the limit
      panel.
    - D drops the rule of A that a save with the switch on needs a limit.
      That rule protects nothing: the reservation check refuses a Boat
      start with no limit, and each connection saves a limit.
    - Cost: a save of the limit still sends the switch and the wait, so an
      old tab can still overwrite them.
    - Effort: Small.
  - **E.** Take the limit out of the policy request. That request then saves
    only `enabled` and `wait_seconds`. A new limit operation saves
    `spend_limit_cents`, and a connection still saves the limit too. The
    spend panel calls the new operation.
    - E drops the assumption that the limit travels in the same request as
      the switch and the wait. Its source is the code and
      `crates/ciproxy-api/overflow.md`, not a requirement.
    - Cost: a breaking change of the request, which is acceptable before
      production. A save of the switch still sends the wait.
    - Effort: Medium.
  - **F.** B with a base-version check. Each save sends its changed fields
    and the settings version that it read. The API refuses a save from an
    older version, with a clear reload message. F was option A of this item
    from 8 November 2026. Cost: a migration for the version, new dashboard
    copy, and a new refusal, which needs the user's consent.
    Effort: Medium.
- **Left out:**
  - Whole-policy last write wins, the current behaviour: a lost spend limit
    costs money.
  - A made-up default limit: it is the current workaround, and "Spend
    limit" in `docs/protocol/overflow.md` rejects a default limit.
  - Overflow off by default until the first connection, which turns it on:
    a save of the wait still needs a limit. The connection would also undo
    a saved off state.
  - A refusal of switch and wait saves before a connection: it fails the
    goal, because the off state still cannot be saved.
  - A switch on the page of a workspace with no Boat connection: not a fix
    of the API rule. Before Boat connects, the switch changes no start, so
    such a switch is a new feature for the `brainstorming` skill.
- **Wider fix:** B, changed from F, the former option A of this item. Each
  save sends only the fields that it changes. So a new policy field cannot
  block other saves, and no save writes back a field that it did not
  change. A, C, and D depend on which field blocks the save. E prevents the
  problem only for the limit.
- **Previous examples:** None found. The two defects of this item share one
  cause.
- **Recommendation:** B, changed from F. One request change removes both
  defects: the tie to the limit for every field, and the write-back of the
  fields that another Admin changed. F also refuses a save from an older
  version, but a new refusal needs the user's consent. D is the Small fix
  for the spend-limit tie alone.
- **Action:** fix after merge, apply B, because it needs a backend and
  dashboard change outside the Milestone 22 scope. The spend-limit tie is
  Low and Rare, because only direct API callers meet it.
