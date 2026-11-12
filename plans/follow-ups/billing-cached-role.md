# Cached billing actions outlive the payer role

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-08, from `conductor/m21-jobs-savings-and-billing-ui`,
  pull request #53, review finding 9, with a problem that its finding check
  found in the workspace banner.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `ts/app/src/state/billing-flows.ts:110`,
  `continuablePayment`, `ts/app/src/screens/billing/plan-usage-screen.tsx:99`,
  `ts/app/src/screens/billing/add-ons-panel.tsx:88`,
  `ts/app/src/view/billing-plans.ts:85`,
  `ts/app/src/view/banner.ts:52`, and
  `ts/app/src/components/billing/workspace-banner.tsx:32`.
- **Reason:** The stale action needs a demotion while the same document
  keeps a payer page and its next billing read is slow, which is an unusual
  timing, and the fix costs the same later.

- **Terms:**
  - **Payer:** a person whose current workspace role is Admin or Billing.
  - **Billing cache:** the last successful billing page that this browser
    document keeps for one workspace.
  - **Effective payer permission:** the current session role is a payer
    role, and the API permission of the page allows the action.
- **Background:**
  - `docs/protocol/billing.md:87-99`: only an Admin or a Billing person
    chooses a tier, changes the add-ons, pays, or opens the portal. Everyone
    else sees "Ask an Admin or a Billing person" in place of each action.
  - Each page visit reads the session again, and the session store keeps
    its old view until the reply arrives, at
    `ts/app/src/state/session.ts:87`. A billing read keeps the cached page
    until its new answer arrives. The billing caches are keyed by the
    services and the workspace only, at `ts/app/src/state/billing.ts:55`
    and `:73`.
  - So after a fresh session says Viewer, these controls still trust the
    `can_pay` of the older payer page:
    - `continuablePayment` returns the saved Stripe URL without the current
      role, at `billing-flows.ts:110-121`, and Continue payment opens it
      directly.
    - Plan and usage takes `canPay` from `page.status.can_pay`, at
      `plan-usage-screen.tsx:99` and `:110`, although it receives the
      current role.
    - The Plans cards take their payer actions from the cached `can_pay`,
      at `billing-plans.ts:85`.
    - The add-ons panel shows its editor from the cached `can_pay`, at
      `add-ons-panel.tsx:88`.
    - The workspace banner keeps the payer action of its last read, such as
      Update card, at `banner.ts:52-58`, on every workspace page until its
      read returns, and also after a failed read.
  - The shared payment gate already reads the current session role, at
    `billing.ts:157`, so no new payment starts from a stale page.
  - The server refuses each dashboard payer request of a Viewer, at
    `crates/ciproxy-api/src/handlers/checkout.rs:33` and
    `crates/ciproxy-api/src/handlers/billing_call.rs:25`. Only the saved
    Stripe link opens without a role check of the server.
  - A browser proof held `GET /billing` after a role change to Viewer. The
    fresh session said Viewer, and the page still showed one Continue
    payment link.
- **Problem:** A cached billing page keeps its payer controls after the
  session says that the person is no longer a payer, until the billing read
  returns.
- **Impact if no action:** A Viewer sees payer actions against the rule of
  `billing.md`, and Continue payment opens the saved Stripe URL with no
  role check. A stable role never meets the defect.
- **Root question:** How does every billing action, on every workspace
  page, show only while the API still lets the person pay?
- **Options:**
  - **A.** Require the current payer role before the billing pages return
    or show Continue payment. Keep the page permission check. Add the
    role-change regression. Effort: Small.
  - **B.** Derive one effective payer permission from the current session
    role and the API permission of the page. Use it for every cached
    billing control: the Plans cards, Plan and usage, the add-ons panel,
    the notices, the saved payment continuation, and the workspace banner.
    Keep the generated values unchanged. Never remove the saved payment
    link because a role changed. Add the role-change regression. Effort:
    Small.
  - **C.** Invalidate the billing caches when the authorization of the
    workspace changes, and require new page data before actions return.
    This adds cache lifecycle work and can discard local edits. Effort:
    Medium.
  - **D.** Show each payer action only after a successful billing read of
    the current visit. Add a checked state to the page and banner stores,
    and show neither the action nor the ask text while a store is
    unchecked. D needs no client copy of the role rule, and it still holds
    if `can_pay` later depends on more than the role. Cost: each payer
    action shows late after each page change, also in the banner. Effort:
    Medium.
- **Left out:**
  - Waiting for the billing answer, because the role mismatch stays
    visible meanwhile.
  - Trusting the current role alone, because it ignores an API permission
    that denies the action.
  - Clearing the saved Stripe URL, because it destroys recovery data and
    fixes no other cached control.
  - A payer-only server route for the open payment, because it checks the
    role only when the person continues, and every other cached control
    still shows.
  - A payer permission in the session reply, because it needs Rust and
    generated API changes, and B gets the same result from the role that
    the session already gives.
  - A loading state on each visit until the billing read returns, because
    it is a variant of D that hides every fact of the page for every role.
- **Wider fix:** B. One effective permission makes the loss of the current
  role constrain every cached billing action at once, also in the banner.
- **Previous examples:** None found. `ts/app/_tests_/billing-resume.test.ts`
  protects only a newly read page of a person who cannot pay.
- **Related:**
  - `plans/follow-ups/billing-portal-feedback.md` adds a shared portal
    control. It takes its visibility from the effective permission of B,
    outside the control.
  - `plans/follow-ups/saved-payment-lifetime.md` orders the removal of the
    same saved link. B gates only the display and adds no removal.
  - `plans/follow-ups/stripe-return-workspace.md`, same code: its B and C
    change the Continue payment controls that B of this item changes. If
    one of them lands, land it with B of this item, so that one
    continuation value carries both the payer permission and the
    workspace. Its recommended A changes none of these files.
- **Recommendation:** B. One effective permission keeps each API
  restriction and the saved URL, with no wait for a person whose role did
  not change. D covers more cases, but it delays every payer action on
  every visit.
- **Action:** fix after merge, apply B, because a fresh Viewer session with
  a stale payer page needs an unusual timing of the role change, and the
  fix costs the same later.
