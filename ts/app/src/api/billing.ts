// The billing routes that read a page or change no payment: Plan and usage,
// Plans, the checkout review, the checkout return, the billing portal, and
// Move to Free. Each payment request goes through billing-payment.ts only.

import type { operations } from "../../../api/index.ts";

import type { CheckoutReturn, PlanSelection } from "./billing-types.ts";
import {
  parseCheckoutReturnPage,
  parseCheckoutReview,
  parsePlanUsagePage,
  parsePlansPage,
  parsePortalLink,
} from "./billing-parse.ts";
import type { ApiClient } from "./client.ts";
import { parseSuccess } from "./parse-session.ts";
import { routePath, withQuery } from "./route-path.ts";

/** The choice that a review asks the API to price. */
export type ReviewRequest = Required<
  NonNullable<operations["checkout_review"]["parameters"]["query"]>
>;

/** The review request of one exact plan choice. */
export function reviewRequest(selection: PlanSelection): ReviewRequest {
  return {
    tier: selection.tier,
    start_blocks: selection.add_ons.start_blocks,
    machines: selection.add_ons.machines,
  };
}

export function createBillingEndpoints(api: ApiClient) {
  const params = (workspaceId: string) => ({ workspace_id: workspaceId });
  return {
    usage: (workspaceId: string) =>
      api.read(
        routePath("/workspaces/{workspace_id}/billing", params(workspaceId)),
        parsePlanUsagePage,
      ),
    plans: (workspaceId: string) =>
      api.read(
        routePath("/workspaces/{workspace_id}/plans", params(workspaceId)),
        parsePlansPage,
      ),
    /** Price one exact choice. Every query value is explicit. */
    review: (workspaceId: string, request: ReviewRequest) =>
      api.read(
        withQuery(
          routePath(
            "/workspaces/{workspace_id}/checkout/review",
            params(workspaceId),
          ),
          {
            tier: request.tier,
            start_blocks: String(request.start_blocks),
            machines: String(request.machines),
          } satisfies Record<keyof ReviewRequest, string>,
        ),
        parseCheckoutReview,
      ),
    /** Reconcile the exact Checkout session that Stripe returned. */
    checkoutReturn: (workspaceId: string, sessionId: string) => {
      const body: CheckoutReturn = { session_id: sessionId };
      return api.change(
        "POST",
        routePath(
          "/workspaces/{workspace_id}/checkout/return",
          params(workspaceId),
        ),
        parseCheckoutReturnPage,
        body,
      );
    },
    portal: (workspaceId: string) =>
      api.change(
        "POST",
        routePath(
          "/workspaces/{workspace_id}/billing/portal",
          params(workspaceId),
        ),
        parsePortalLink,
      ),
    moveToFree: (workspaceId: string) =>
      api.change(
        "POST",
        routePath(
          "/workspaces/{workspace_id}/billing/move-to-free",
          params(workspaceId),
        ),
        parseSuccess,
      ),
  };
}

export type BillingEndpoints = ReturnType<typeof createBillingEndpoints>;
