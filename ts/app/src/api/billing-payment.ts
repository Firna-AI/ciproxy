// Every payment request of the dashboard goes through `submitPayment`: a new
// paid plan through Stripe Checkout, a move between paid plans on the saved
// card, and saved add-ons. Each request sends the exact choice of the price
// that the person saw, the review of a plan or the quote of add-ons, and
// nothing that the person edited after it. A later quote contract changes
// this module only.

import type {
  CheckoutReviewPage,
  PaymentOutcome,
  PlanAddOns,
  PlanOffer,
  PlanSelection,
} from "./billing-types.ts";
import { parsePaymentOutcome } from "./billing-parse.ts";
import type { ApiClient, ApiResult } from "./client.ts";
import { routePath } from "./route-path.ts";

export type PaymentSubmission =
  /** A plan from its review: Checkout from Free, else a paid change. */
  | { kind: "plan"; review: CheckoutReviewPage }
  /** Add-ons of the current paid plan, at the price that the page showed. */
  | { kind: "add-ons"; quote: PlanOffer };

/** Whether two plan choices are the same tier with the same add-ons. */
export function sameSelection(a: PlanSelection, b: PlanSelection): boolean {
  return (
    a.tier === b.tier &&
    a.add_ons.start_blocks === b.add_ons.start_blocks &&
    a.add_ons.machines === b.add_ons.machines
  );
}

/** The exact choice that a submission pays for. */
export function submittedSelection(
  submission: PaymentSubmission,
): PlanSelection {
  const selection =
    submission.kind === "plan"
      ? submission.review.choice.selection
      : submission.quote.selection;
  return {
    tier: selection.tier,
    add_ons: {
      start_blocks: selection.add_ons.start_blocks,
      machines: selection.add_ons.machines,
    },
  };
}

export function submitPayment(
  api: ApiClient,
  workspaceId: string,
  submission: PaymentSubmission,
): Promise<ApiResult<PaymentOutcome>> {
  const params = { workspace_id: workspaceId };
  const selection = submittedSelection(submission);
  if (submission.kind === "add-ons") {
    const addOns: PlanAddOns = selection.add_ons;
    return api.change(
      "PUT",
      routePath("/workspaces/{workspace_id}/billing/add-ons", params),
      parsePaymentOutcome,
      addOns,
    );
  }
  const route = submission.review.prorated
    ? routePath("/workspaces/{workspace_id}/billing/change", params)
    : routePath("/workspaces/{workspace_id}/checkout", params);
  return api.change("POST", route, parsePaymentOutcome, selection);
}
