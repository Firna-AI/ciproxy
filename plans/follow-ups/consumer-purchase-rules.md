# Paid checkout misses consumer purchase rules

- **Severity:** Medium. **Effort:** Medium. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-22, from the branch `docs/legal-pages-firna`, problems
  outside the scope that the check of review finding 5 found.
- **Location:** `ts/app/src/screens/billing/checkout-screen.tsx`.
  `ts/app/src/screens/billing/`. `docs/protocol/billing.md`, "Checkout".
  `ts/site/src/data/legal-terms.ts`, "Plans and payment".
- **Reason:** The fix changes who can buy a paid plan or adds product steps,
  and only the user can choose either.

- **Terms:**
  - **Consumer:** a person who buys for purposes outside their trade,
    business, or profession.
  - **Withdrawal period:** the 14 days in which a consumer can cancel a
    distance contract without a reason.
- **Background:**
  - The terms accept customers in any country and keep the mandatory
    consumer protection of the law where a consumer lives.
  - The terms give consumers the 14-day withdrawal right, with the model
    instructions and a model form. They also say that the checkout asks for
    no early-start request, so a withdrawal gets a full refund. A checkout
    with that request needs a new terms version without that sentence.
  - A paid plan starts at once when Stripe confirms the payment.
  - ciproxy sends no email, and the repository sets no text for the Stripe
    receipt.
- **Problem:** The paid checkout misses three consumer rules:
  - The Consumer Contracts Regulations 2013, regulation 36(1), need an
    express request to start a service inside the withdrawal period. The
    checkout asks for none. Without it, a consumer who withdraws owes
    nothing for the days used.
  - Regulation 16 needs a confirmation on a durable medium, with the
    cancellation information, before the service starts. None exists.
  - Since 19 June 2026, Article 11a of Directive 2011/83/EU needs a
    withdrawal function in the online interface for EU consumers. The
    dashboard has none.
  - Regulation 13(1) needs the withdrawal information before the purchase.
    Stripe Checkout and the dashboard review neither show nor link it. The
    dashboard links the terms only on the sign-in page, which the GitHub
    setup return skips. If that does not count, regulation 31(3) adds 12
    months to the withdrawal period.
  - Regulation 14(4) and (5) need words about payment on the button that
    places a paid order. The button that charges the saved card for a paid
    move says "Move to" and the plan name
    (`ts/app/src/view/billing-checkout.ts`, near line 110). Without those
    words, a consumer is not bound by that order.
- **Impact if no action:** A consumer can withdraw late or owe nothing, and
  a regulator can act on the missing steps.
- **Root question:** How does ciproxy meet the consumer rules for distance
  contracts when a consumer buys a paid plan?
- **Options:**
  - **A.** Keep the current checkout, and handle each withdrawal by hand.
    Effort: Small.
  - **B.** Sell paid plans for business use only. Say so in the terms and on
    the checkout review, and keep the consumer clause as a fallback. It
    changes a dashboard screen, its mockup, and `docs/protocol/billing.md`.
    Effort: Medium.
  - **C.** Meet the rules: an express-request box at checkout, a durable
    confirmation with the cancellation information, and a "Withdraw from
    contract here" function with an acknowledgment and a refund. Effort:
    Large.
  - **D.** Let a merchant of record sell the plans, such as Stripe Managed
    Payments, which then owns the consumer rules. Effort: Medium.
- **Left out:** Keeping out EU consumers by country breaks the rule from the
  conductive terms that customers in any country can use the service.
- **Wider fix:** None, because each option is the whole fix.
- **Previous examples:** None found.
- **Related:** `taxes-after-launch.md`. Its option D, Stripe Managed
  Payments, is option D here, so answer both together.
- **Recommendation:** B. ciproxy serves GitHub organizations, and B removes
  the consumer-only duties at Medium effort. D is better if the tax item
  picks a merchant of record.
- **Action:** ask the user, because B changes who can buy a paid plan, and
  the terms that the user chose name no customer type.
