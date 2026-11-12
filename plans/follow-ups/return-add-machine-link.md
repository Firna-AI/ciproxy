# Add machines after payment opens the list

- **Severity:** Low. **Likelihood:** Common. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-08, from `conductor/m21-jobs-savings-and-billing-ui`,
  pull request #53, a new problem that the check of owner finding 3 found.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `ts/app/src/view/billing-return-page.ts:83` and
  `ts/app/_tests_/billing-return.test.ts:206`.
- **Reason:** The Add machine page and its address belong to the Machines
  pages, which the branch that found the problem does not build.

- **Terms:**
  - **Confirmed result:** the return page after Stripe Checkout, once the
    API confirms the payment.
  - **Add machine page:** the Machines page that adds an own machine. Its
    mockup is `app/machines/add`, `paths.addMachine` at
    `docs/mockups/src/components/paths.ts:72`.
- **Background:**
  - `docs/protocol/security.md:58` and `:69-70`: only an Admin adds own
    machines.
  - The confirmed result gives an Admin "Add machines" as its primary
    action, then "Open plan and usage", at `billing-return-page.ts:76-88`.
    A Billing person gets only "Open plan and usage". The owner chose this
    role rule on 2026-11-08. Its fix kept the Admin actions and their
    addresses as they were.
  - The address is `workspaceHref("machines", workspace.id)`, which gives
    `/machines?workspace=<id>`, the Machines list.
  - Both return mockups open the Add machine page, at
    `docs/mockups/src/screens/billing/checkout-result.tsx:122` and
    `docs/mockups/src/screens/billing/api-return.tsx:85`.
  - In this repository, `/machines` shows "This page is coming soon" in the
    shell, at `ts/app/src/screens/coming-soon.tsx:43`, with the Setup action
    for an Admin. So today the action opens that page for any address.
  - The Machines pages own the addresses of the list, of one machine, and of
    the Add machine page. Search `ts/app/src/view` for their builder of the
    Add machine address, such as `addMachineHref`, before the change.
  - The test "after a confirmed payment, only an Admin gets Add machines",
    at `ts/app/_tests_/billing-return.test.ts:189`, builds the expected
    Admin address with `workspaceHref` at `:206`.
- **Problem:** The Add machines action of an Admin opens the Machines list,
  while both mockups open the Add machine page.
- **Impact if no action:** After the Machines pages land, an Admin who just
  paid needs one more step to add a machine. Today the action opens the
  coming-soon page of Machines.
- **Root question:** Which page does Add machines open after a confirmed
  payment?
- **Options:**
  - **A.** When the Add machine page lands, open it through the address
    builder of the Machines pages. Keep the label, the primary look, the
    order, the Admin-only rule, and Open plan and usage. Change the expected
    Admin address of the test to the same builder. Effort: Small.
  - **B.** Build the Add machine address in the return model before the
    Machines pages land. Cost: a second builder of one address, which must
    guess their query and can drift from it, while the page stays coming
    soon until they land. Effort: Small.
- **Left out:**
  - No change, because the action keeps opening another page than its
    mockups.
  - Hiding Add machines until the Machines pages land, because both
    mockups show it to the Admin, and the role rule of 2026-11-08 keeps it
    for the Admin.
  - The machine step of Setup until the Machines pages land, because no
    mockup opens it from this action, and the address would change twice.
- **Wider fix:** A. The builder of the Machines pages stays the one source
  of their addresses, so this link cannot drift from them.
- **Previous examples:** None found.
- **Recommendation:** A. It opens the page of the mockups through the
  builder that owns its address.
- **Action:** fix after merge, apply A, because the Add machine page and its
  builder belong to the Machines pages, which the branch that found the
  problem does not build.
