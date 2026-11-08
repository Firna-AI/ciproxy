# Stripe returns do not name their workspace

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-08, from `conductor/m21-jobs-savings-and-billing-ui`,
  the handoff of the billing pages.
- **Source:** Implementation.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-billing/src/service/runtime.rs:20`,
  `crates/ciproxy-billing/src/service/checkout.rs:45`,
  `crates/ciproxy-billing/src/service/runtime.rs:107`,
  `ts/app/src/state/billing-context.ts`, and
  `docs/protocol/billing.md:108`.
- **Reason:** The fix is a change of the billing service and its runtime
  configuration, which the UI branch does not own.

- **Terms:**
  - **Redirects:** the struct at `runtime.rs:20` with one `success_url`,
    one `cancel_url`, and one `portal_return_url`.
  - **Tab context:** the workspace that a dashboard tab saves in its own
    tab storage just before it leaves for Stripe Checkout or the Stripe
    portal.
- **Background:**
  - `docs/protocol/billing.md` says that the return page waits a few
    seconds for the confirmation, and then says that Stripe still confirms
    and updates by itself.
  - Every dashboard page names its workspace by an opaque ID in
    `?workspace=<id>`.
  - `Redirects` holds one URL of each kind for all workspaces. Checkout
    copies the success and cancel URLs unchanged at `checkout.rs:45-46`.
    The portal uses `portal_return_url` at `runtime.rs:107`.
  - The success URL holds Stripe's `{CHECKOUT_SESSION_ID}` placeholder, so
    the return names the exact session. The cancel URL and the portal
    return name no session.
  - No runtime composition sets `Redirects` yet. Only
    `crates/ciproxy-billing/examples/stripe_test_mode.rs` builds one.
  - The dashboard uses the tab context for a return without `workspace`:
    `/checkout-return` and `/plans` use the workspace saved before
    Checkout, and `/billing` the one saved before the portal. It never
    uses the last remembered workspace, because other pages change it.
- **Problem:** A return that opens in a tab without the tab context, such
  as another browser, cannot tell its workspace. The return page then says
  "This tab cannot tell the workspace of this payment" and shows no
  outcome.
- **Impact if no action:** That person sees no confirmation of the payment.
  The signed Stripe event still confirms the payment, and Plan and usage
  shows it later.
- **Root question:** How does each Stripe return name its exact workspace?
- **Options:**
  - **A.** The redirect templates get a workspace placeholder. The server
    fills it with the workspace of the command when it builds the
    Checkout and the portal requests, for the success, the cancel, and the
    portal return URLs. The tab context stays as a fallback. Effort: Small.
- **Left out:**
  - The last remembered workspace or any other guess, because a payment
    return must never open another workspace.
  - A workspace that the browser supplies without a server-built URL, for
    the same reason.
  - A server lookup of the workspace of an exact Checkout session, because
    it covers only the success return. The cancel and the portal returns
    name no session. A also covers the success return, with less work.
  - Only the tab context of today, because a return in another browser
    still cannot name its workspace.
- **Wider fix:** A. Every return then carries a workspace that the server
  chose.
- **Previous examples:** None found.
- **Recommendation:** A. It covers all three returns with one small
  service change, before any runtime sets the URLs.
- **Action:** fix after merge, apply A, because the change is in the
  billing service and its configuration, outside the UI branch.
