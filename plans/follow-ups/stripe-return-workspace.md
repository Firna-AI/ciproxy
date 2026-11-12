# Stripe returns do not name their workspace

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-08, from `conductor/m21-jobs-savings-and-billing-ui`,
  the handoff of the billing pages. Extended on 2026-11-08 with finding 12
  of the second review round of that branch, which has the same cause.
- **Source:** Implementation and Review.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-billing/src/service/runtime.rs:20`,
  `crates/ciproxy-billing/src/service/checkout.rs:45`,
  `crates/ciproxy-billing/src/service/runtime.rs:107`,
  `crates/ciproxy-billing/examples/stripe_test_mode.rs:77`,
  `ts/app/src/state/billing-context.ts:11`,
  `ts/app/src/state/billing-flows.ts:52`,
  `ts/app/src/state/billing-flows.ts:91`,
  `ts/app/src/screens/billing/usage-notices.tsx:19`,
  `ts/app/README.md:183`, and `docs/protocol/billing.md:108`.
- **Reason:** The fix is a change of the billing service and its runtime
  composition, which the UI branch does not own.

- **Terms:**
  - **Redirects:** the struct at `runtime.rs:20` with one `success_url`,
    one `cancel_url`, and one `portal_return_url`.
  - **Tab context:** the workspace that a dashboard tab saves in its own
    tab storage just before it leaves for Stripe Checkout or the Stripe
    portal.
  - **Checkout:** the Stripe Checkout page where a workspace pays for its
    move from Free to a paid plan.
  - **Continue payment:** the action of Plan and usage and of Plans that
    opens the saved Stripe address of an unfinished payment again. It sends
    no payment request.
- **Background:**
  - `docs/protocol/billing.md` says that the return page waits a few
    seconds for the confirmation, and then says that Stripe still confirms
    and updates by itself.
  - Every dashboard page names its workspace by an opaque ID in
    `?workspace=<id>`. A query with `workspace` always wins over the tab
    context, at `ts/app/src/screens/billing/billing-route.tsx:58-62`.
  - `Redirects` holds one URL of each kind for all workspaces. Checkout
    copies the success and cancel URLs unchanged at `checkout.rs:45-46`.
    The portal uses `portal_return_url` at `runtime.rs:107`.
  - The success URL holds Stripe's `{CHECKOUT_SESSION_ID}` placeholder, so
    the return names the exact session. The cancel URL and the portal
    return name no session.
  - The dashboard serves the Checkout return at `/checkout-return`. The
    page reads the exact session from `session_id`, at
    `ts/app/src/screens/billing/checkout-return-screen.tsx:175`, and the
    workspace from `workspace`, at `billing-route.tsx:58`. The dashboard
    has no `/billing/return` route, so that address shows "Page not found".
    `ts/app/README.md:183-187` asks for the success URL
    `/checkout-return?session_id={CHECKOUT_SESSION_ID}` on the dashboard
    origin.
  - No runtime composition on this branch sets `Redirects`. The Stripe
    test-mode example at `stripe_test_mode.rs:77` and the API test setup at
    `crates/ciproxy-api/tests/http/m18_support.rs:59-60` build these three
    URLs on an origin:

    ```text
    <origin>/billing/return?session_id={CHECKOUT_SESSION_ID}
    <origin>/plans
    <origin>/billing
    ```

  - A runtime composition that was proposed on 8 November 2026, in a new
    file `crates/ciproxy-control-plane/src/startup_billing.rs`, copies the
    same three URLs. Its origin is the public URL of the control plane.
    With it, every success return shows "Page not found".
  - The dashboard uses the tab context for a return without `workspace`:
    `/checkout-return` and `/plans` use the workspace saved before
    Checkout, and `/billing` the one saved before the portal. It never
    uses the last remembered workspace, because other pages change it.
  - The tab keeps one Checkout context and one portal context, at
    `ts/app/src/state/billing-context.ts:11-16`. It keeps one saved Stripe
    address for each workspace, at
    `ts/app/src/state/billing-open-payment.ts:22-24`.
  - Only the payment store saves the Checkout context, in its `leave`, just
    before a new Checkout opens, at `billing-flows.ts:52-55`. Continue
    payment is a plain same-tab link with no click handler, at
    `usage-notices.tsx:19-30`. Plans shows the same control at
    `ts/app/src/screens/billing/plans-screen.tsx:93`. It saves no context.
  - The return store clears the one Checkout context when a return ends as
    confirmed or unfinished, whatever its workspace, at
    `billing-flows.ts:91-97`.
  - The API keeps `payment_pending` true while a Checkout stays open, at
    `crates/ciproxy-api/src/handlers/page_context.rs:177`. So the page of a
    workspace still offers Continue payment after another workspace opened
    its Checkout.
  - The billing service refuses the Checkout session of one workspace for
    another, at `checkout.rs:100-104`. The API sends that refusal as a 403,
    at `crates/ciproxy-api/src/handlers/billing_call.rs:24-26`. The return
    page then says "This payment is not for this workspace", at
    `ts/app/src/view/billing-return-page.ts:173-181`. The signed Stripe
    event still confirms the payment of the right workspace.
  - These cases give a missing or wrong tab context:
    - Another browser: the return opens in a tab that saved no context.
    - Two unfinished Checkouts in one tab: open and leave the Checkout of
      workspace A, then do the same for workspace B. Open A again, and
      select Continue payment. The Checkout of A opens, but the context
      still names B.
    - Back to an older Stripe page: after the Checkout of B saved the
      context, the Back button of the browser opens the Checkout of A
      again. No dashboard click happens, so no dashboard code can save the
      context first.
    - An ended return: the return of B ends as confirmed or unfinished, and
      it clears the one context. A later Continue payment of A leaves with
      no context.
- **Problem:** Each Stripe return names no workspace, so the return page
  depends on the one tab context. In each case above, that context is
  missing or names another workspace. A success return then shows no
  outcome or a false workspace error, and a cancel opens the Plans page of
  another workspace or the landing page.
- **Impact if no action:** After a successful payment, that person sees no
  confirmation or a false workspace error, and a cancel can open another
  workspace. The signed Stripe event still confirms the payment, and Plan
  and usage of the right workspace shows it later.
- **Root question:** How does each Stripe return name its exact workspace?
- **Options:**
  - **A.** The redirect templates get a workspace placeholder, such as
    `{WORKSPACE_ID}`. The server fills it with the workspace of the command
    when it builds the Checkout and the portal requests, for the success,
    the cancel, and the portal return URLs. It leaves Stripe's
    `{CHECKOUT_SESSION_ID}` for Stripe. The runtime composition uses the
    routes that the dashboard serves, on the dashboard origin. In a Rust
    `format!` string, each brace doubles.

    ```text
    <origin>/checkout-return?workspace={WORKSPACE_ID}&session_id={CHECKOUT_SESSION_ID}
    <origin>/plans?workspace={WORKSPACE_ID}
    <origin>/billing?workspace={WORKSPACE_ID}
    ```

    A test of the billing service checks that the success and the cancel
    URLs of a Checkout, and the portal return, name its workspace. The
    Stripe test-mode example, the API test setup, and
    `ts/app/README.md:183-187` change to the same routes. The tab context
    stays as a fallback for a configured URL without the placeholder. The
    startup does not reject such a URL, because a new rejection needs a
    user decision. Effort: Medium.
  - **B.** Pass the workspace ID into Continue payment, and save its
    Checkout context before the same-tab link opens the saved Checkout, on
    Plan and usage and on Plans. Add a regression that starts with the
    saved context of another workspace. B covers a second Checkout and an
    ended return in one tab. It covers neither a Back step to an older
    Stripe page nor a return in another browser. Effort: Small.
  - **C.** Give the first payment and each Checkout continuation one shared
    navigation helper. The helper requires the workspace ID and the Stripe
    address, and it saves the context before the tab leaves. Use it for
    Continue payment and for the open-Checkout links of the payment and
    return results. Keep the real link addresses, and add the regression of
    B. C has the limits of B, and the helper must know each dashboard exit
    to Stripe. Effort: Small.
- **Left out:**
  - The last remembered workspace or any other guess, because a payment
    return must never open another workspace. The last workspace that the
    person visited is such a guess, because normal navigation changes it.
  - A workspace that the browser supplies without a server-built URL, for
    the same reason.
  - A server lookup of the workspace of an exact Checkout session, because
    it covers only the success return. The cancel and the portal returns
    name no session. A also covers the success return, with less work.
  - Only the tab context of today, because a return in another browser
    still cannot name its workspace.
  - A check of the returned session against each workspace that the tab
    saved, because a cancel return names no session.
  - A tab context for each Checkout session, because the `open` answer of
    the API names only a Stripe URL. Such a key needs a new wire field or a
    parse of the Stripe URL format, which is no contract. A is better on
    every point.
  - A Continue payment that sends the payment request again, so that the
    payment store saves the context. It breaks the rule that continuing
    sends no request, which `ts/app/_tests_/billing-resume.test.ts:64`
    protects. Also, the payment gate stays closed while a payment waits.
  - A context that a billing page saves when it opens with an unfinished
    Checkout. It breaks the rule that normal navigation cannot retarget a
    return, which `ts/app/_tests_/billing-routes.test.ts:17` protects.
  - An end of the Checkout of the other workspace, because separate
    workspaces can each have an unfinished payment.
  - No Continue payment, because the unfinished-payment rule needs a way to
    finish that exact payment.
  - A startup rejection of a configured URL without the workspace
    placeholder, because a new rejection needs a user decision. The tab
    context covers such a URL.
- **Wider fix:** A. Every return then carries a workspace that the server
  chose, also after a Back step, in another browser, and when a dashboard
  exit to Stripe changes shape.
- **Previous examples:** The Stripe test-mode example and the API test
  setup build the success URL with `/billing/return`, which the dashboard
  does not serve, and the proposed runtime composition copies it.
  `ts/app/_tests_/billing-routes.test.ts:17` protects the context against
  normal navigation, but no test starts from the saved Checkout context of
  another workspace.
- **Related:**
  - `plans/follow-ups/billing-cached-role.md`, same code: its B changes
    `continuablePayment` and the Continue payment controls that B and C of
    this item change. If B or C lands, land it with that fix, so that one
    continuation value carries both the payer permission and the
    workspace. A changes none of these files.
  - `plans/follow-ups/saved-payment-lifetime.md`, same code: its A changes
    the `send` of `createPagePayment`, beside the `leave` that C replaces.
    A continuation saves only the context, never a link record, so the
    order of that fix stays correct. A changes none of these files.
  - `plans/follow-ups/billing-portal-feedback.md`, same component: its B
    replaces the portal action of `ResultAction` in
    `ts/app/src/screens/billing/payment-result.tsx`. C routes the
    open-Checkout link of the same component. If both land, the portal
    control can use the exit helper of C. A changes none of these files.
  - None of these items waits for this item, and this item waits for none
    of them.
- **Recommendation:** A. It binds all three returns to their workspace on
  the server, also after a Back step, in another browser, and after
  another return cleared the tab context. B and C save the context only on
  a click in the dashboard.
- **Action:** fix after merge, apply A, because the change is in the
  billing service and its runtime composition, outside the UI branch, and
  the fix costs the same later.

The proposed runtime composition, as a permalink:
<https://github.com/Firna-AI/ciproxy/blob/eb11d7c2a64c160c20fc331060b4a7c377884cdf/crates/ciproxy-control-plane/src/startup_billing.rs#L19-L23>
