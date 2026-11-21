# Invoice read failure blocks later paid reviews

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-08, from pull request #39, branch
  `conductor/m18-composition-pages-stripe-hook-paid-review`, review finding
  5 after its third fix round.
- **Location:** `crates/ciproxy-billing/src/service/payment_invoice.rs:38`,
  `crates/ciproxy-billing/src/service/paid_review.rs:57`, and
  `crates/ciproxy-billing/src/stripe/wire/invoice.rs:8`.
- **Reason:** The defect needs a rare partial Stripe outage and blocks only a
  later review, and the pull request had used its approved fix rounds.

- **Terms:**
  - **Quote:** the stored selection, amounts, and time that authorize one
    move between the paid Team and Business tiers.
  - **PaymentWork:** the stored journal that tracks moved credit and the
    invoice comparison of one paid move, in
    `crates/ciproxy-store-interface/src/billing/payments.rs`.
  - **Expanded invoice:** the complete invoice object that Stripe returns
    inside a subscription response when the request expands it.
- **Background:**
  - On 8 November 2026 the user chose that the payment of a move up carries
    the earlier pending credits and charges that its review counted. ciproxy
    also compares each paid move's invoice with its quote. Every option keeps
    both rules.
  - Before the paid change, billing removes the original pending items and
    records the moved value in `PaymentWork`. The exact paid invoice then
    settles that value.
  - While a journal holds moved value, `money_held` in
    `crates/ciproxy-billing/src/service/payment_work.rs:45` is true. A new
    paid review then returns `PaymentConfirming`
    (`crates/ciproxy-billing/src/service/paid_review.rs:57`). The API maps it
    to HTTP 409 with the Problem code `conflict`.
  - The subscription read expands `latest_invoice`
    (`crates/ciproxy-billing/src/stripe/http/commands.rs:57`). Stripe's
    expand returns the complete linked object.
  - `payment_invoice` in
    `crates/ciproxy-billing/src/service/payment_invoice.rs:31` finds the
    invoice of the paid change. When the expanded invoice carries this
    payment's quote and command metadata, line 38 still requests the same
    invoice again, at line 43.
  - The expanded `Invoice` type drops the invoice customer
    (`crates/ciproxy-billing/src/stripe/wire/invoice.rs:8`). Only
    `RetrievedInvoice` keeps it (`:102`). The ownership check at
    `payment_invoice.rs:58` compares that customer, the subscription, and the
    quote and command metadata.
  - The test `invoice_read_outage_confirms_payment_and_recovers_its_exact_invoice_after_restart`
    in `crates/ciproxy-control-plane/tests/dashboard/operator_invoice_tests.rs:43`
    proves that the payment stays confirmed and that maintenance recovers
    after the outage. It requests no new review during the outage.
  - The test `audit_write_failure_confirms_payment_allows_review_and_recovers_one_mismatch_alert`
    in `crates/ciproxy-control-plane/tests/dashboard/operator_write_tests.rs:19`
    covers a later review only after the settlement saves.
- **Problem:** When the second invoice request fails, `inspect_payment`
  returns that error before it settles the journal. `money_held` stays true,
  so each new paid review returns HTTP 409 with `conflict`. The facts that
  the request fetches are already in the expanded invoice.
- **Impact if no action:** The payment and the paid tier stay confirmed.
  During a partial outage, Stripe invoice reads fail while subscription reads
  and previews still work. Then the customer cannot review another paid move.
  The block ends after an invoice read works again and the billing
  maintenance pass, which runs each minute, settles the journal.
- **Root question:** How can a later paid review stay available once ciproxy
  has exact proof that the earlier move's invoice is paid?
- **Options:**
  - **A.** On an invoice-request failure, use the verified expanded paid
    invoice to settle the credit. Preserve and check its customer field.
    Cover both the stored-ID and marker branches. Extend the outage test to
    request another review. Effort: Small.
  - **B.** Use one verified invoice representation for expanded and
    separately retrieved invoices. Check customer, subscription, quote, and
    command identities together. Settle credit from that receipt before
    diagnostic work. Cover both the stored-ID and marker branches. Extend
    the existing outage test. Effort: Medium.
  - **C.** Save the verified payment receipt atomically with the confirmed
    billing state. Run invoice comparison through maintenance. This changes
    the store interface and more reconciliation paths. Effort: Medium.
  - **D.** On a failed separate read, settle from the matched expanded invoice.
    Use its subscription's verified customer. This drops the separate
    invoice-owner check and leaves two ownership rules. The Stripe guarantee
    for that shortcut was not confirmed. Effort: Small.
- **Left out:**
  - Clearing the credit hold because the tiers are equal, because that lacks
    exact payment proof.
  - Waiting for the repeated request to recover, because that keeps the
    defect.
  - A bounded read retry, because a long partial outage still blocks review
    and it adds time while the billing lock is held.
  - Settlement from a signed event alone, because the expanded object is
    newer and serves payment, webhook, and maintenance paths.
- **Wider fix:** B. A common verified receipt removes the dependency on a
  second request for facts that Stripe already returned.
- **Previous examples:** The base renewal path also rereads an expanded
  invoice in `crates/ciproxy-billing/src/service/renewal.rs:54` through
  `crates/ciproxy-billing/src/service/observations.rs:61`.
- **Related:** Preserve the stored quote, proration time, observed period,
  payment-side carried value, and diagnostic no-rollback rules. Every option
  keeps those requirements. No new public error code or parser entry is needed.
- **Recommendation:** B. Shared ownership checks best protect the financial
  settlement across the subscription, webhook, and maintenance paths.
- **Action:** fix after merge, apply B, because this rare defect affects only
  a later review, and the authorized fix rounds are complete.

Source: [Stripe expand](https://docs.stripe.com/expand)
