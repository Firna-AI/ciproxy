# New adjustments do not invalidate quotes

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-17, from the final review of 17 November 2026 of the
  branch `calummoore/ci-proxy-load-balancer`, finding 503.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-billing/src/service/payment_transfer.rs:39`,
  `crates/ciproxy-billing/src/service/payment_transfer.rs:198`,
  `crates/ciproxy-billing/src/stripe/http/preview.rs:60`, and
  `docs/protocol/billing-paid-moves.md:25`.
- **Reason:** The charge stays equal to the review, and only the required
  refusal is missing, in a rare order of add-on changes.

- **Terms:**
  - **Move up:** a paid change to a higher tier or to more add-ons. It
    charges the difference today.
  - **Quote:** a stored review of a paid move. It fixes the selection, the
    amounts, and the proration time.
  - **Pending item:** a credit or a charge that Stripe keeps for a later
    invoice, such as the credit of an extra machine that the customer
    removed.
  - **Offset:** the pending items that the review of a move up counted.
    The payment carries them, so that Stripe computes the reviewed charge.
- **Background:**
  - `docs/protocol/billing-paid-moves.md:25-28` requires a refusal. ciproxy
    refuses to start a move when the subscription, its tier and add-ons,
    its billing day, its period, its earlier credits and charges, or the
    credit balance of the customer changed after the review. The page then
    asks the customer to review the move again.
  - A decision at `:36-38`: each review needs its own confirmation.
    Rejected: a move that starts by itself after the new review.
  - A decision at `:56-60`: the payment of a move up carries the credits
    and charges that its review counted. Rejected: taking them out of the
    review, computing the charge in ciproxy, invoicing them before the
    move, and invoicing each add-on change at once. The user chose this on
    8 November 2026, with an alert when the paid invoice differs from the
    quote.
  - `validate_offset` at
    `crates/ciproxy-billing/src/service/payment_transfer.rs:39-54` checks
    the credit balance. Then it checks only that each counted item is still
    pending. It accepts added items. When the quote has no offset, it reads
    no pending items.
  - The check just before the paid update, at `payment_transfer.rs:198-214`,
    reads the subscription and the balance again, but not the pending
    items.
  - An add-on change can create pending items through Stripe's proration.
    Raising and then restoring an add-on quantity keeps the subscription
    the same, but it leaves new pending items.
  - Stripe leaves pending items that already exist out of the invoice of
    such an update. The section "Existing pending invoice items" of
    [Stripe's pending updates page][stripe-pending] says so for updates
    with `pending_if_incomplete` and with `always_invoice`. No live payment
    has shown it yet.
  - The test fake at
    `crates/ciproxy-billing/tests/_tests_/fake_credits.rs:128-136` computes
    the charge from the proration, the offset, and the balance only.
  - So today the charge still equals the review, and each new item reaches
    a later invoice once.
  - A quote stays usable after another billing command commits.
    `crates/ciproxy-store-pg/src/billing/quotes.rs:35-72` deletes only
    expired quotes. The add-on path at
    `crates/ciproxy-billing/src/service/commands.rs:177-178` does not touch
    quotes.
  - `save_command` at `crates/ciproxy-store-pg/src/billing/session.rs:87`
    saves each billing command: add-on changes, cancellation, Checkout, and
    command updates. A move with an unknown quote gets `ReviewRequired`, at
    `commands.rs:44`.
- **Problem:** An old quote can start a move up after the earlier credits
  and charges changed. `validate_offset` accepts added pending items, so
  the required refusal and the new review do not happen.
- **Impact if no action:** The move starts on a review whose credit and
  charge context changed, against the contract. The charge itself stays
  right.
- **Root question:** Which changes to the earlier credits and charges must
  stop an old quote from starting a move up, and where does ciproxy notice
  them?
- **Options:**
  - **A.** Repeat the preview at the original proration time. Compare its
    pending sources and amounts before the payment starts. Add the
    regression case. Cost: five to nine more Stripe calls at each start of
    a move up. A cannot run in the check before the paid update, because
    the counted items are deleted by then. Effort: Small.
  - **B.** Permit new pending items, and leave them for a later invoice.
    Update the contract to say so. Cost: it changes a requirement, and it
    depends on a Stripe exclusion that no live payment has shown.
    Effort: Small.
  - **C.** Store the complete set of pending sources in the quote, also an
    empty set. Use one source builder for the review and for the payment
    check. The check before the paid update reads the pending items again
    too. Cost: one or two more Stripe reads. Effort: Medium.
  - **D.** Cancel the open quotes of a workspace when ciproxy saves another
    billing command.
    - Delete the unused quotes, where `command_id` is null, in
      `save_command`.
    - A later move then gets `ReviewRequired`, because its quote is
      unknown.
    - D adds no Stripe read, and it covers every change that ciproxy makes.
    - It misses pending items from outside ciproxy, such as a credit that
      staff add in the Stripe dashboard.
    - Effort: Small.
- **Left out:**
  - Comparing only the totals: it misses changed items whose amounts cancel
    each other.
  - A block on add-on changes while a quote is open: D gives the same
    protection, and add-on changes stay available.
  - A move that prices itself again with the new items: it breaks the
    decision that each review needs its own confirmation.
  - Taking the pending items out of the review: the decision of 8 November
    2026 rejects it.
  - A guard in the dashboard: the server refusal already sends the page to
    a new review, and a guard in the page does not stop a direct API call.
- **Wider fix:** C. It stores the complete reviewed context, not only the
  counted items. A is broader, because it compares the outcome of the
  review and also catches a change outside the pending items. But A costs
  five to nine Stripe calls at each start.
- **Previous examples:** None found in `origin/main` or in the earlier
  review reports.
- **Recommendation:** C. It refuses every change to the credits and
  charges, also one from outside ciproxy, for one or two more Stripe reads.
  A costs more Stripe calls, D misses items from outside ciproxy, and B
  depends on a Stripe exclusion that no live payment has shown.
- **Action:** fix after merge, apply C, because the charge stays right, and
  only the required refusal is missing in a rare order of changes.

[stripe-pending]: https://docs.stripe.com/billing/subscriptions/pending-updates
