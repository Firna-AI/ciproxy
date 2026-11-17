# Old reads can remove a saved payment link

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-08, from `conductor/m21-jobs-savings-and-billing-ui`,
  pull request #53, review finding 8, with a problem of its finding check:
  a failed Check again removes a link that still works. The final review of
  17 November 2026 of the branch `calummoore/ci-proxy-load-balancer` found
  the same defect again as finding 601. Its check added options D, E, and
  F, and F became the recommendation.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `ts/app/src/state/billing.ts:42`, `forgetEnded`,
  `ts/app/src/state/page-store.ts`, in `load` of `createPageStore`,
  `ts/app/src/state/billing-flows.ts:46`, in `createPagePayment`, and
  `ts/app/src/state/billing-open-payment.ts:51`, `clearOpenPayment`.
- **Reason:** An older read must stall until after a payment answers, which
  is an unusual timing, and the fix costs the same later.

- **Terms:**
  - **Saved payment link:** the exact Stripe page of a payment, a Checkout
    or the payment page of a paid change, that this tab saves for Continue
    payment. `ts/app/src/state/billing-open-payment.ts` keeps one record for
    each workspace in the storage of the tab.
  - **Page store:** `createPageStore` in `ts/app/src/state/page-store.ts`.
    The Usage store and the Plans store each keep one, with their own
    counter of the latest read.
- **Background:**
  - The rule of the record, at `billing-open-payment.ts:1-7`: each server
    outcome replaces the record, and a read that shows no pending payment
    removes it, so the tab never offers an ended payment.
  - `forgetEnded` removes the link when an answer says that no payment is
    pending, at `billing.ts:42-53`. It runs in the read of the Usage store,
    at `:66`, and of the Plans store, at `:89`.
  - The removal runs before `load` of the page store compares the answer
    with the latest read of its store and drops an older answer.
  - So an answer from before a payment can remove the link that the payment
    saved later. Three timings do so:
    - A page read is held while the payment answers, and then ends.
    - The Plans store still loads its first read when the payment answers,
      and the payment does not read Plans again.
    - An open outcome leaves for Stripe with no read after it, and an older
      read ends before the page unloads.
  - A payment request also removes the link when it starts: "A new or
    repeated payment request replaces any saved URL.", at
    `billing-flows.ts:45-46`. So a failed Check again of a payment page
    removes a link that the server still accepts, and a page change then
    leaves no way to continue.
  - Each outcome already replaces or clears the record, and a current read
    with no pending payment clears it. So the removal at the start of a
    request has no reason left.
  - The server counts an open paid change as pending, at
    `crates/ciproxy-api/src/handlers/page_context.rs:177-183`, and it
    commits the change before it answers with the link. So a read that
    starts after the last write of the link sees the pending payment.
  - Both page reads are database reads. They normally answer long before a
    person clicks and the Stripe call ends, so normal use does not meet the
    defect.
  - The tab record is the only source of the link today. No API returns
    the link of an open payment.
  - The server identifies a waiting Checkout by its billing command:
    `CheckoutProgress.command_id`, at
    `crates/ciproxy-store-interface/src/billing/commands.rs:53`.
- **Problem:** An older read, or a failed repeat of a payment request,
  removes a link that still belongs to a waiting payment. The page then
  says only that Stripe is confirming a payment, with no Continue payment.
- **Impact if no action:** After a page change or a reload, the person
  cannot open the Stripe page that this tab received. No new payment can
  start while the payment waits, so the wait ends only when Stripe confirms
  or ends the payment.
- **Root question:** How does this tab keep the saved Stripe link of a
  payment for exactly as long as that payment waits on the server, in any
  order of reads, payment requests, and answers? The rule has two sides:
  never lose the link of a waiting payment, and never offer an ended
  payment.
- **Options:**
  - **A.** Keep a generation for each workspace in the record module, and
    advance it at every write that saves a link. Each read captures the
    generation before it starts, and its removal applies only while the
    generation is the same. The page stores need no hook for accepted
    answers. A payment request no longer removes the link when it starts.
    The link stays until a real outcome replaces it, or a current
    successful read shows no pending payment. Usage and Plans use the same
    mechanism. Add the delayed-answer regression and the failed Check again
    regression. Effort: Small.
  - **B.** Serialize the billing reads and mutations of a workspace in one
    queue, so that older reads end before a payment starts. As in A, a
    payment request no longer removes the link when it starts, because the
    queue alone does not keep the link through a failed Check again. A
    payment can then wait behind a slow read. Effort: Medium.
  - **C.** Recover open payments from the server, not from the tab record,
    as `plans/follow-ups/open-payment-resume.md` proposes. A page then gets
    the link of a waiting payment from the server again, also after a
    failed Check again. It needs an API extension. Effort: Medium.
  - **D.** Keep the tab record, and get a missing link from the server.
    - Build the payer-only resume route of
      `plans/follow-ups/open-payment-resume.md`, its option B.
    - A page calls the route while a fresh read says pending with
      `can_pay`, and the tab holds no link.
    - This recovers a lost link in this tab, after a failed Check again,
      and in other tabs and browsers.
    - Cost: a new API route, a Stripe read of the invoice URL, generated
      types, a protocol update, and tests. It adds behaviour at the API
      boundary, so it needs the user.
    - Effort: Medium.
  - **E.** Key the saved record to its payment.
    - The API adds the identity of the waiting payment, its billing command
      ID, to `PlanStatus` and to each `PaymentOutcome`.
    - The tab saves the identity with the link. It offers the link only
      when the current read names the same payment.
    - No read and no request deletes the record. The next outcome replaces
      it.
    - Cost: new fields in two wire types, in Rust and in the generated
      TypeScript, and tests. It covers this tab only, and it is a boundary
      change.
    - Effort: Medium.
  - **F.** Apply A with these exact rules:
    - Keep one generation for each workspace in the record module, in
      memory. Memory is enough, because the reads in flight end with the
      document.
    - Each write of the record advances the generation.
    - Readers get only the conditional removal. The module exports no
      unconditional clear.
    - A payment request no longer removes the link when it starts.
    - The link stays until an outcome replaces it, or until a read that
      started after the last write shows no pending payment.
    - Add a delayed-read regression for both stores, and a regression for a
      failed Check again.
    - Effort: Small.
- **Left out:**
  - A generation of the page store alone, because it misses old reads of
    the other store and the old reads after an outcome left for Stripe.
  - Keeping links forever in its plain form, because the tab must never
    offer an ended payment. E is the form that keeps this rule.
  - A second read before the link ends, because A is better on every
    point. A sends no extra request, and the second read can still carry an
    older snapshot.
  - One store for Usage and Plans, because A covers both stores without
    the merge.
  - No new payment while a billing read runs, because a read that starts
    during the payment request can still carry an older snapshot. B is the
    complete form of this approach.
  - A comparison of the saved record at the start and at the end of each
    read, with no generation. It is the same approach as A and F, and it
    fails when the same link is saved again.
  - The display rule `continuablePayment` alone. It already shows the link
    only after a fresh pending read. The defect deletes the stored link
    that it reads.
- **Wider fix:** F. One generation in the record module controls every
  removal. So no older read from either store, in any timing, and no failed
  request can remove a newer link. D is wider still, because the server
  becomes the source in every browser, but D needs the user.
- **Previous examples:** None found of this order of removal.
- **Related:**
  - `plans/follow-ups/open-payment-resume.md` covers another browser that
    has no link. This item keeps the link that this tab received. Neither
    item waits for the other.
  - `plans/follow-ups/billing-cached-role.md` gates who sees the same saved
    link. A role change gates only the display, and it must not remove the
    link.
  - `plans/follow-ups/explicit-session-refresh-order.md`, the same class in
    other code: an older read that acts after a newer one. Its read rule
    orders the reads of one store only. So it cannot order a read against a
    payment write or against the other billing store, and neither item
    closes the other.
- **Recommendation:** F, changed from A. F is A with exact rules: the
  generation lives in memory, each write advances it, and readers get only
  the conditional removal. So no caller can remove a newer link without
  the check, at the same Small cost. Like A, F covers both stores and every
  writer, with no extra read and no change of the stored format. D is
  wider, but it is a boundary change.
- **Action:** fix after merge, apply F, because the defect needs an unusual
  timing, and the fix costs the same later.
