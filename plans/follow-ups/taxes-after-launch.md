# ciproxy collects no tax on its payments

- **Severity:** Medium. **Effort:** Medium. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-04, from the branch
  `calummoore/ci-proxy-load-balancer`, the user's answer on taxes.
- **Location:** `docs/protocol/billing.md`, "Tiers".
- **Reason:** On 4 November 2026 the user chose no tax at launch "for now",
  and an agent call of 22 November 2026 kept that decision in force, so the
  tax method waits until a later sale needs one.

- **Terms:**
  - **Stripe Tax:** a Stripe feature that adds sales tax or VAT at checkout
    from the address of the customer. It costs 0.5% of each payment.
  - **Merchant of record:** the company that sells to the customer in law,
    and that owes the sales tax and the VAT of the sale.
  - **Stripe Managed Payments:** a Stripe service that makes Stripe the
    merchant of record. It costs 3.5% of each payment on top of the Stripe
    fees, and it became generally available in April 2026.
- **Background:**
  - ciproxy sells three tiers in US dollars: Free, Team at $49 a month, and
    Business at $299 a month. Stripe Checkout takes every payment.
  - On 4 November 2026 the user decided that ciproxy collects no tax at
    launch, "for now".
  - On 22 November 2026, the user left the decisions of the open legal
    items to the agent. Under that delegation, an agent call kept the
    decision of 4 November 2026 in "Tiers" of `docs/protocol/billing.md` in
    force: ciproxy collects no tax for now. This item stays open. It waits
    until a later sale needs a tax method.
  - Under the same delegation, an agent call of 22 November 2026 made the
    paid plans for business use only. "Checkout" in
    `docs/protocol/billing.md` and the terms of service say so, and each
    checkout review with a pay button says so too. The consumer clause of
    the terms stays as a fallback. So a merchant of record is no longer
    needed for the consumer rules of a sale.
  - Many countries and US states make a seller of software collect tax from
    local customers. The duty depends on where the seller is registered and
    on its sales in each place.
  - The seller is Firna Ltd, a company registered in England and Wales.
    The terms of service on the site give its UK VAT number, GB191373695.
    A VAT-registered UK seller usually charges UK VAT on its sales to
    customers in the UK. The terms of service also say that prices exclude
    tax and that ciproxy collects no tax.
- **Problem:** The payments carry no tax, although some customers' places
  can make the seller owe tax on them.
- **Impact if no action:** The company can owe tax that it did not
  collect, and pay it from its own revenue.
- **Root question:** Who owes the sales tax and the VAT of each payment,
  and how does the price show it?
- **Options:**
  - **A.** Keep no tax. Effort: Small.
  - **B.** Turn on Stripe Tax. Prices say "plus tax", and checkout asks for
    a tax ID, so business buyers get the reverse charge. The company
    registers, collects, and files. Effort: Medium.
  - **C.** B, with prices that include tax. Effort: Medium.
  - **D.** Stripe Managed Payments. Stripe collects and files the tax in
    over 80 countries. Customers see Link, a Stripe brand, as the seller.
    Stripe must accept the business, in a listed country. Effort: Medium.
  - **E.** ciproxy keeps its own tax rates and gives them to Checkout. It
    then carries each error. Effort: Large.
- **Left out:** Another merchant of record, such as Paddle, because Stripe
  takes every payment, and D gives the same result on Stripe.
- **Wider fix:** None, because this is one setting.
- **Previous examples:** None found.
- **Recommendation:** D. It removes every tax registration and filing for
  about 3% more of each payment. B is better if the company already
  collects tax with Stripe Tax.
- **Action:** ask the user, because only the user knows the tax
  registrations of the company, and the options trade fees against tax
  work.
