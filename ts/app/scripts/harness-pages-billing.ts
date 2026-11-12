import type { components } from "../../api/index.ts";
import {
  checkoutReview,
  fixtureCheckoutId,
  parseAddOns,
  parseCheckoutReturn,
  parsePaidChange,
  parseSelection,
  paymentOutcome,
  reviewSelection,
} from "./fixtures-billing-payment.ts";
import { usageForRole } from "./fixtures-billing-usage.ts";
import { workspaceId } from "./fixtures-core.ts";
import {
  pageError,
  pageReply,
  settlePayment,
  type PageReply,
  type PageRoute,
  type PagesFixtureState,
} from "./harness-pages-state.ts";
import type { CapturedBody } from "./harness-pages-facts.ts";

export function serveBillingPage(
  route: PageRoute,
  query: URLSearchParams,
  value: unknown,
  pages: PagesFixtureState,
  role: components["schemas"]["WorkspaceRole"],
): PageReply {
  const usage = usageForRole(pages.usage, role);
  if (route === "checkout/review") {
    const selected = reviewSelection(query, usage.offer.selection);
    if (selected === undefined)
      return pageError(400, "invalid_request", "Choose a plan and add-ons.");
    if (
      selected.tier === "free" ||
      selected.tier === usage.status.selection.tier ||
      usage.status.payment_pending ||
      usage.status.operation_in_progress
    ) {
      return pageError(
        409,
        "conflict",
        "Choose a different plan after the current payment ends.",
      );
    }
    if (pages.failures.includes(route))
      return pageError(503, "unavailable", "The review is unavailable.");
    const review = checkoutReview(selected, usage);
    if (review.quote) pages.quotes.set(review.quote.id, review);
    return pageReply(review);
  }
  const paidChange = parsePaidChange(value);
  const quote =
    paidChange === undefined
      ? undefined
      : pages.quotes.get(paidChange.quote_id);
  const selected =
    route === "billing/change"
      ? quote?.choice.selection
      : parseSelection(value);
  const addOns = parseAddOns(value);
  const returned = parseCheckoutReturn(value);
  let captured: CapturedBody = null;
  if (route === "checkout") captured = selected ?? { invalid: true };
  if (route === "billing/change") captured = paidChange ?? { invalid: true };
  if (route === "billing/add-ons") captured = addOns ?? { invalid: true };
  if (route === "checkout/return")
    captured =
      returned === undefined ? { invalid: true } : { session_id: "[redacted]" };
  const fail = (
    status: number,
    code: components["schemas"]["ProblemCode"],
    message: string,
  ) => pageError(status, code, message, captured);
  if (pages.failures.includes(route))
    return fail(503, "unavailable", "Billing is unavailable. Try again.");
  if (route === "billing/portal") {
    if (usage.status.selection.tier === "free")
      return fail(409, "conflict", "The workspace has no paid subscription.");
    return pageReply(
      {
        url: `/__fixture/stripe/portal?workspace=${workspaceId}`,
      } satisfies components["schemas"]["PortalLink"],
      captured,
    );
  }
  if (route === "billing/move-to-free") {
    if (usage.status.payment_pending || usage.status.operation_in_progress)
      return fail(409, "conflict", "A billing operation is in progress.");
    if (usage.status.selection.tier !== "free")
      pages.usage.status.cancel_at = pages.usage.status.period_end;
    return pageReply(
      { ok: true } satisfies components["schemas"]["Success"],
      captured,
    );
  }
  if (route === "checkout/return") {
    if (returned === undefined)
      return fail(400, "invalid_request", "Supply the Checkout session.");
    if (returned.session_id !== fixtureCheckoutId)
      return fail(404, "not_found", "Checkout not found.");
    settlePayment(pages, pages.returned);
    return pageReply(
      {
        outcome: paymentOutcome(pages.returned),
        banner: usageForRole(pages.usage, role).banner,
      } satisfies components["schemas"]["CheckoutReturnPage"],
      captured,
    );
  }
  if (usage.status.payment_pending || usage.status.operation_in_progress)
    return fail(409, "conflict", "A payment is already in progress.");
  if (route === "billing/change") {
    if (paidChange === undefined)
      return fail(400, "invalid_request", "Supply the reviewed quote.");
    if (pages.reviewRequired || quote === undefined)
      return fail(409, "review_required", "Review this change again.");
  }
  if (route === "billing/add-ons") {
    if (addOns === undefined)
      return fail(400, "invalid_request", "Supply the add-on quantities.");
    if (usage.status.selection.tier === "free")
      return fail(409, "conflict", "Choose a paid plan for add-ons.");
    pages.target = { tier: pages.usage.status.selection.tier, add_ons: addOns };
  } else {
    if (selected === undefined)
      return fail(400, "invalid_request", "Supply a plan and add-ons.");
    if (
      selected.tier === "free" ||
      selected.tier === usage.status.selection.tier
    )
      return fail(409, "conflict", "Choose a different paid plan.");
    if ((route === "checkout") !== (usage.status.selection.tier === "free"))
      return fail(
        409,
        "conflict",
        "Use the payment route for the current plan.",
      );
    pages.target = selected;
  }
  const outcome = pages.payment;
  settlePayment(pages, outcome);
  return pageReply(paymentOutcome(outcome), captured);
}
