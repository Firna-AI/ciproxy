# Three protocol docs pass the length target

- **Severity:** Low. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-22, from the branch
  `conductor/ciproxy-terms-acceptance-and-business-checkout`, the
  documentation milestone, implementation item 3.
- **Location:** `docs/protocol/billing.md`, "Subscription states" and
  "Failed payments". `docs/protocol/workspaces.md`, "Inactive
  organizations". `docs/protocol/agent.md`, "Platforms" and "The
  virtualization check".
- **Reason:** Each split moves inbound links in up to six other files, and
  the branch that touched these docs only applied five legal follow-up
  items.

- **Terms:**
  - **Protocol doc:** a page under `docs/protocol/` that holds the
    requirements and the decisions in force for one area.
  - **Length target:** the rule of the `documentation` skill that a
    protocol doc stays at about 250 lines.
  - **Split page:** a linked page that takes some sections of a protocol
    doc, such as `docs/protocol/billing-paid-moves.md`. It opens with one
    short paragraph that says what it states and links back, and it ends
    with "Related docs".
- **Background:**
  - The `documentation` skill of the shared skills says: "Keep each doc
    short, about 250 lines. When you touch a doc that passes the target,
    split it into linked pages in the same change."
  - On 22 November 2026, the branch
    `conductor/ciproxy-terms-acceptance-and-business-checkout` applied five
    legal follow-up items. It split `docs/protocol/security.md`, which had
    317 lines, into `security.md`, `terms-acceptance.md`,
    `security-isolation.md`, and `security-data.md`.
  - The same branch touched three docs that already passed the target:
    - `docs/protocol/billing.md`: 251 lines before the branch, 261 after.
      The branch added the business-use requirement to "Checkout".
    - `docs/protocol/workspaces.md`: 259 lines before, 263 after. The
      branch added a rule for linked installation entries and repointed
      links.
    - `docs/protocol/agent.md`: 268 lines before, 269 after. The branch
      changed one line of "Related docs".
  - On 22 November 2026, the orchestrator agent, under the user's
    delegation, chose to split these three docs after the merge.
- **Problem:** Three protocol docs pass the length target, and the branch
  that touched them did not split them.
- **Impact if no action:** A reviewer reads longer pages to check each
  requirement. The next change that touches one of these docs must split
  it first.
- **Root question:** How do the billing, workspace, and agent requirements
  stay at about 250 lines for each page?
- **Options:**
  - **A.** Move one self-contained topic of each doc into a split page.
    Update each inbound link in the same change. Effort: Medium.
    - `billing.md`: move "Subscription states" (21 lines) and "Failed
      payments" (21 lines) into a new
      `docs/protocol/billing-subscriptions.md`. `billing.md` then has
      about 220 lines. Update the links to `billing.md#subscription-states`
      in `docs/architecture.md` near line 62,
      `docs/protocol/app-removal.md` near lines 91 and 131, and
      `docs/protocol/metering.md` near line 61. Update the links to
      `billing.md#failed-payments` in `docs/architecture/data-flow.md`
      near line 76 and `docs/protocol/metering.md` near line 99.
    - `workspaces.md`: move "Inactive organizations" (59 lines) into a new
      `docs/protocol/workspaces-inactive.md`. `workspaces.md` then has
      about 207 lines. Update the links to
      `workspaces.md#inactive-organizations` in
      `docs/protocol/app-removal.md` near line 67 and
      `docs/protocol/metering.md` near line 91. Update the links to
      `#inactive-organizations` inside `workspaces.md`, near lines 23, 35,
      and 111.
    - `agent.md`: move "Platforms" (30 lines) and "The virtualization
      check" (24 lines) into a new `docs/protocol/agent-platforms.md`.
      `agent.md` then has about 218 lines. Update the links to
      `agent.md#platforms` in `docs/architecture.md` near line 72,
      `docs/protocol/agent-images.md` near line 25,
      `docs/protocol/agent-install.md` near line 139,
      `docs/protocol/job-routing.md` near line 44, and
      `docs/protocol/labels.md` near line 13. Update the links to
      `agent.md#the-virtualization-check` in
      `docs/protocol/agent-install.md` near line 133 and
      `docs/protocol/metering.md` near line 71. Update the link to
      `#the-virtualization-check` inside `agent.md`, near line 113.
    - For each new page, add a key to
      `docs/mockups/src/components/related-docs.ts`. Add the key to the
      `relatedDocs` of each mockup entry that shows the moved rules: the
      payment entries under `docs/mockups/src/entries/app/billing/payments/`
      and `app/billing/current-contract/payments/`, the entries under
      `app/settings/organizations/inactive/`, and the add-machine entries
      under `app/onboarding/add-machine/` and `app/machines/add/`.
    - Update the "Related docs" of each split doc and of its new page.
  - **B.** Move other sections, which have more inbound links: "Checkout"
    of `billing.md` (62 lines, 6 inbound links), "Add an organization" of
    `workspaces.md` (99 lines with its subsections, 6 inbound links), and
    "Drain and remove" of `agent.md` (42 lines, 12 inbound links). Effort:
    Medium.
- **Left out:** Change nothing, because it fails the goal of the length
  target. Shorter wording in place of a split, because the rules forbid
  compacting text to stay under a limit, and each requirement stays.
- **Wider fix:** None. A length check in the gate would detect the class,
  but a new check needs the user's approval, and the `documentation` skill
  already holds the rule.
- **Previous examples:** `docs/protocol/billing-paid-moves.md` came from
  `billing.md`. `docs/protocol/job-routing-capacity.md` and
  `docs/protocol/job-routing-examples.md` came from
  `docs/protocol/job-routing.md`. `docs/protocol/security.md` split into
  four pages on 22 November 2026.
- **Recommendation:** A. Each proposed page holds one topic with few
  inbound links, so each split moves the fewest links.
- **Action:** fix after merge, apply A, because the splits move links in
  other docs, outside the five legal items of the branch that found the
  problem.
