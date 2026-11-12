# Add-on scenes show unreachable payment results

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-08, from `conductor/m21-jobs-savings-and-billing-ui`,
  a new problem that the check of finding 11 of its second review round
  found. The same check removed finding 11, because the answer that the
  finding needs never comes.
- **Source:** Review.
- **Category:** Mockup.
- **Location:**
  `docs/mockups/src/entries/app/billing/current-contract/add-ons/states.mockup.tsx:61`,
  `docs/mockups/src/entries/app/billing/current-contract/add-ons/states.mockup.tsx:83`,
  `docs/mockups/src/entries/app/billing/current-contract/pending/states.mockup.tsx:42`,
  `docs/mockups/src/screens/billing/api-add-ons.tsx:27`, and
  `docs/mockups/src/screens/billing/checkout-review.tsx:97`.
- **Reason:** The check found it in the last review fix round of the
  branch, which takes no new work, and no customer sees these scenes.

- **Terms:**
  - **Current-contract scene:** a mockup scene that shows a moment with
    only the facts that the API gives, under
    `docs/mockups/src/entries/app/billing/current-contract/`.
  - **Known-data scene:** a mockup scene of the fixture story, which shows
    each page with every fact.
  - **Add-on save:** Save add-ons on Plan and usage. It sends new add-on
    quantities for the current paid plan with
    `PUT /api/workspaces/<id>/billing/add-ons`.
  - **Move up:** a move from Team to Business on the Checkout review page of
    the dashboard. It sends `POST /api/workspaces/<id>/billing/change`, and
    Stripe charges the saved card at once.
  - **Payment answer:** the `PaymentOutcome` of the API: `open`,
    `confirming`, `confirmed`, `unfinished`, or `payment_required` with an
    optional Stripe page.
- **Background:**
  - The add-on intent keeps the confirmed tier, at
    `crates/ciproxy-billing/src/service/commands.rs:27-28` and `:89`. So an
    add-on save is no upgrade, at `commands.rs:124`.
  - A change that is no upgrade sends `create_prorations` and
    `allow_incomplete` to Stripe, at
    `crates/ciproxy-billing/src/stripe/forms.rs:47-66`. Stripe makes no
    invoice and applies the quantities at once.
  - The completion check then gives `Applied`, at
    `crates/ciproxy-billing/src/service/completion.rs:47`, and the command
    returns it at `commands.rs:158-160`. The API sends it as `confirmed`,
    at `crates/ciproxy-api/src/handlers/billing_views.rs:109`.
  - During a move up that waits, an add-on save gets `PaymentConfirming`,
    which the API sends as a 409. The test at
    `crates/ciproxy-billing/tests/_tests_/pending_context_tests.rs:28-37`
    shows it.
  - Only a move up sends `always_invoice` and `pending_if_incomplete`, at
    `forms.rs:47-66`. Only its open subscription-update invoice gives a
    Stripe page, at `commands.rs:161-169`. When its charge does not go
    through, the answer is `payment_required`, with or without that page.
  - `docs/protocol/billing.md:80-81` says that a saved add-on change takes
    effect at once, and that Stripe prorates the difference.
    `docs/protocol/billing.md:149-158` says that a move up charges the
    saved card at once. A failed charge keeps the tier and offers to try
    again or to update the card. The Stripe payment page opens when the
    bank asks for a confirmation. `crates/ciproxy-billing/README.md:30-32`
    and `:39` say the same.
  - The billing tests agree. The test at
    `crates/ciproxy-billing/tests/_tests_/changes_tests.rs:87` gets
    `Applied` with `create_prorations` for a saved add-on change. Each
    `PaymentRequired` case in the billing tests is a move to Business.
  - The product words each payment answer in
    `ts/app/src/view/billing-outcome.ts`. A confirmed add-on save says
    "Your add-ons are saved" and "Stripe charges the difference for the
    rest of this period." `payment_required` with a Stripe page says "Your
    payment is not finished", with "Open the Stripe page" and "Check
    again". Without a page, it says "Stripe could not charge your card",
    with "Update card". The product handles each answer that the wire type
    allows, so the product needs no change.
  - Three current-contract scenes show an add-on save with an answer that
    the API never sends to it:
    - `add-ons/bank`, "A payment to finish on its Stripe page", at
      `add-ons/states.mockup.tsx:61-81`: Save add-ons gets
      `payment_required` with a Stripe page.
    - `add-ons/card`, "The card fails", at `add-ons/states.mockup.tsx:83-97`:
      Save add-ons gets a failed card.
    - `pending/bank-continue`, "Continue a payment on its Stripe page after
      a reload", at `pending/states.mockup.tsx:42-58`. Its story says that
      "the payer saved 1 block of starts, and the payment waited on the
      Stripe page of its unpaid invoice". Its screen, the pending notice
      with Continue payment, is right for a move up.
  - The Checkout review mockup shows only the results of a payment whose
    answer the page did not get. Its `ReviewOutcome` at
    `docs/mockups/src/screens/billing/checkout-review.tsx:97-100` is
    "unknown", "current", or "waits". The notices "invoice" and "card"
    already exist in `docs/mockups/src/screens/billing/api-outcome.tsx`.
    The add-on panel accepts "invoice", "card", and "unknown", at
    `docs/mockups/src/screens/billing/api-add-ons.tsx:27`.
  - The current-contract `checkout` page shows four screens, `add-ons`
    five, and `pending` five. A page shows at most five screens, so a sixth
    needs a linked sub-page.
  - No known-data scene shows an add-on save with a Stripe page or a failed
    card. The known-data `payments` scenes show a failed renewal, which the
    banner flow reaches.
  - The model test at `ts/app/_tests_/billing-outcome.test.ts:250-268`
    builds the same answer for an add-on save by hand. The browser harness
    gives one payment answer to every payment route, at
    `ts/app/scripts/harness-pages-billing.ts:126-128`. So its
    `m21-payment-required` scenario, a move to Business at
    `ts/app/scripts/harness-pages-scenarios.ts:234-238`, also gives that
    answer to an add-on save. Both test how the product words an answer
    that the wire type allows. No server sends it to an add-on save.
- **Problem:** Two current-contract scenes show an add-on save with a
  result that the API never gives it, and a third tells a story that the
  API cannot produce. The current-contract scenes must show only what the
  API gives, as `docs/mockups/README.md:89-91` says.
- **Impact if no action:** A reader of the spec expects a Stripe page or a
  failed card after Save add-ons, and builds or reviews a state that no
  customer reaches. A review of this branch already raised a false finding
  from these scenes.
- **Root question:** How do the current-contract billing scenes show only
  the results that the API can send on each route?
- **Options:**
  - **A.** Move each result to the route that sends it, and correct the
    story:
    - Remove `bank` and `card` from the `add-ons` page. In their place,
      show the answer that an add-on save gets: "Your add-ons are saved",
      with "Stripe charges the difference for the rest of this period."
    - Show the "invoice" and "card" results on Checkout after a move up.
      Widen `ReviewOutcome` to accept them. Put them on a linked sub-page
      of `checkout`, because that page would pass five screens.
    - Narrow the outcome type of the add-on panel to the answers of an
      add-on save.
    - Tell the story of `pending/bank-continue` as a move up whose payment
      waits on its Stripe page. Keep its screen.
    - Update the descriptions, and the billing sentences of
      `docs/mockups/README.md:94-103` where they name these pages. Keep
      every known-data scene unchanged. Run the mockup check, and look at
      the changed pages.
    - Effort: Small.
  - **B.** Remove the `bank` and `card` scenes, and correct the story of
    `pending/bank-continue`, with no new scene. The results of a move up
    then have no current-contract scene, although the product shows them
    on Checkout. Effort: Small.
- **Left out:**
  - No change, because the current-contract scenes must show only what the
    API gives.
  - A note in the two scene descriptions that the API does not send this
    answer today, because the scenes would still show a state that the
    current API cannot reach.
  - Add-on saves that charge at once, so that the scenes become reachable.
    That changes the billing behaviour that `docs/protocol/billing.md:80-81`
    states, and this task keeps the backend as it is.
  - Changes to the model test and the browser harness, because they test
    how the product words an answer that the wire type allows. This task
    is the mockup.
  - Only two options are real, because each other approach keeps a scene
    that the API cannot reach, or changes the backend.
- **Wider fix:** A. The narrowed outcome type of the add-on panel cannot
  hold a Stripe page or a failed card, so no add-on scene can show them
  again.
- **Previous examples:** The removed review finding, about a notice that
  stays after a finished add-on payment, came from the same unreachable
  answer. No earlier change of these scenes was found.
- **Recommendation:** A. It keeps each example on the route that reaches
  it, and the narrowed type keeps the add-on scenes on the answers of the
  API. B loses the move-up results from the current-contract scenes.
- **Action:** fix after merge, apply A, because the check found it in the
  last review fix round of the branch, and no customer sees these scenes.
