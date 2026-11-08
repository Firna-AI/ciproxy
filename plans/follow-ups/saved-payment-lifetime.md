# Old reads can remove a saved payment link

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-08, from `conductor/m21-jobs-savings-and-billing-ui`,
  pull request #53, review finding 8, with a problem of its finding check:
  a failed Check again removes a link that still works.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `ts/app/src/state/billing.ts:42`, `forgetEnded`,
  `ts/app/src/state/billing-page.ts:50`, in `load`,
  `ts/app/src/state/billing-flows.ts:44`, in `createPagePayment`, and
  `ts/app/src/state/billing-open-payment.ts:51`, `clearOpenPayment`.
- **Reason:** An older read must stall until after a payment answers, which
  is an unusual timing, and the fix costs the same later.

- **Terms:**
  - **Saved payment link:** the exact Stripe page of a payment, a Checkout
    or the payment page of a paid change, that this tab saves for Continue
    payment. `ts/app/src/state/billing-open-payment.ts` keeps one record for
    each workspace in the storage of the tab.
  - **Page store:** `createPageStore` in `ts/app/src/state/billing-page.ts`.
    The Usage store and the Plans store each keep one, with their own
    counter of the latest read.
- **Background:**
  - The rule of the record, at `billing-open-payment.ts:1-7`: each server
    outcome replaces the record, and a read that shows no pending payment
    removes it, so the tab never offers an ended payment.
  - `forgetEnded` removes the link when an answer says that no payment is
    pending, at `billing.ts:42-53`. It runs in the read of the Usage store,
    at `:64`, and of the Plans store, at `:83`.
  - The removal runs before `load` compares the answer with the latest read
    of its page store and drops an older answer, at `billing-page.ts:50`.
  - So an answer from before a payment can remove the link that the payment
    saved later. Three timings do so:
    - A page read is held while the payment answers, and then ends.
    - The Plans store still loads its first read when the payment answers,
      and the payment does not read Plans again.
    - An open outcome leaves for Stripe with no read after it, and an older
      read ends before the page unloads.
  - A payment request also removes the link when it starts: "A new or
    repeated payment request replaces any saved URL.", at
    `billing-flows.ts:43-44`. So a failed Check again of a payment page
    removes a link that the server still accepts, and a page change then
    leaves no way to continue.
  - The server counts an open paid change as pending, at
    `crates/ciproxy-api/src/handlers/page_context.rs:177-183`, and it
    commits the change before it answers with the link. So a read that
    starts after the last write of the link sees the pending payment.
- **Problem:** An older read, or a failed repeat of a payment request,
  removes a link that still belongs to a waiting payment. The page then
  says only that Stripe is confirming a payment, with no Continue payment.
- **Impact if no action:** After a page change or a reload, the person
  cannot open the Stripe page that this tab received. No new payment can
  start while the payment waits, so the wait ends only when Stripe confirms
  or ends the payment.
- **Root question:** How does this tab keep a saved payment link exactly as
  long as its payment waits?
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
- **Left out:**
  - A generation of the page store alone, because it misses old reads of
    the other store and the old reads after an outcome left for Stripe.
  - Keeping links forever, because the tab must never offer an ended
    payment.
  - A second read before the link ends, because A is better on every
    point. A sends no extra request, and the second read can still carry an
    older snapshot.
  - One store for Usage and Plans, because A covers both stores without
    the merge.
  - No new payment while a billing read runs, because a read that starts
    during the payment request can still carry an older snapshot. B is the
    complete form of this approach.
- **Wider fix:** A. One generation that lives with the record stops every
  older answer, from either store and in every timing.
- **Previous examples:** None found of this order of removal.
- **Related:**
  - `plans/follow-ups/open-payment-resume.md` covers another browser that
    has no link. This item keeps the link that this tab received. Neither
    item waits for the other.
  - `plans/follow-ups/billing-cached-role.md` gates who sees the same saved
    link. A role change gates only the display, and it must not remove the
    link.
- **Recommendation:** A. A generation with the record covers both stores and
  every writer, with no extra read and no change of the stored format.
- **Action:** fix after merge, apply A, because the defect needs an unusual
  timing, and the fix costs the same later.
