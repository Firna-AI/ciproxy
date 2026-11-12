// Generated-type values of the billing contract for the billing tests. Each
// builder gives one complete response that a test changes in place.

import type {
  CheckoutReviewPage,
  PageBanner,
  PageBannerView,
  PaidQuoteReference,
  PlanOffer,
  PlanSelection,
  PlanUsagePage,
} from "../src/api/billing-types.ts";
import type { WorkspaceRole } from "../src/api/schema.ts";
import { apiPrefix } from "../src/api/client.ts";
import { routePath } from "../src/api/route-path.ts";
import { session, workspace, workspaceId } from "../scripts/fixtures-core.ts";
import { appWith, response, type Reply } from "./support.ts";

export const billingApiPaths = {
  usage:
    apiPrefix +
    routePath("/workspaces/{workspace_id}/billing", {
      workspace_id: workspaceId,
    }),
  change:
    apiPrefix +
    routePath("/workspaces/{workspace_id}/billing/change", {
      workspace_id: workspaceId,
    }),
  review:
    apiPrefix +
    routePath("/workspaces/{workspace_id}/checkout/review", {
      workspace_id: workspaceId,
    }),
};

/**
 * Services whose session gives the fixture person `role` in the fixture
 * workspace, then the replies. The session read is setup, so `calls` lists
 * only the requests after it.
 */
export async function signedInWith(role: WorkspaceRole, ...replies: Reply[]) {
  const made = appWith(
    response(session({ workspaces: [workspace({ role })] })),
    ...replies,
  );
  await made.app.session.ensure();
  made.calls.splice(0, made.calls.length);
  return made;
}

export function selection(
  tier: PlanSelection["tier"],
  startBlocks = 0,
  machines = 0,
): PlanSelection {
  return { tier, add_ons: { start_blocks: startBlocks, machines } };
}

/** An offer whose values a test sets; nothing here computes a price. */
export function offer(
  chosen: PlanSelection,
  monthlyCents: string,
  includedStarts: string,
  includedMachines: string,
): PlanOffer {
  return {
    selection: chosen,
    monthly_cents: monthlyCents,
    included_starts: includedStarts,
    included_machines: includedMachines,
  };
}

export const freeOffer = offer(selection("free"), "0", "2000", "1");
export const teamOffer = offer(selection("team"), "4900", "25000", "3");
export const paidQuote: PaidQuoteReference = {
  id: "023e4567-e89b-12d3-a456-426614174000",
  expires_at: "2026-11-06T12:30:00Z",
};

/** Plan and usage on Free, with Team as the offer and nothing pending. */
export function usagePage(
  overrides: Partial<PlanUsagePage> = {},
): PlanUsagePage {
  return {
    status: {
      selection: selection("free"),
      subscription: "none",
      period_start: "2026-11-01T00:00:00Z",
      period_end: "2026-12-01T00:00:00Z",
      cancel_at: null,
      payment_pending: false,
      operation_in_progress: false,
      can_pay: true,
    },
    current: freeOffer,
    offer: teamOffer,
    starts: "1240",
    machines: "1",
    organizations: "1",
    expected_starts: "18600",
    starts_over: "0",
    machines_over: "0",
    organizations_over: "0",
    crossing_day: "2026-11-04",
    banner: null,
    ...overrides,
  };
}

/** The usage page with a payment that Stripe still confirms. */
export function pendingUsagePage(): PlanUsagePage {
  const page = usagePage();
  page.status.payment_pending = true;
  page.status.operation_in_progress = true;
  return page;
}

type ReviewValues = Partial<Omit<CheckoutReviewPage, "prorated" | "quote">>;

/** The review of a move from Free to a paid choice. */
export function review(
  choice: PlanOffer = teamOffer,
  overrides: ReviewValues = {},
): CheckoutReviewPage & { prorated: false; quote: null } {
  return {
    choice,
    today_cents: choice.monthly_cents,
    billing_day: 6,
    prorated: false,
    quote: null,
    period_end: "2026-12-01T00:00:00Z",
    banner: null,
    ...overrides,
  };
}

/** A paid change keeps the quote that permits its payment. */
export function paidReview(
  choice: PlanOffer = teamOffer,
  overrides: ReviewValues = {},
): CheckoutReviewPage & { prorated: true; quote: PaidQuoteReference } {
  return { ...review(choice, overrides), prorated: true, quote: paidQuote };
}

export function bannerView(banner: PageBanner | null = null): PageBannerView {
  return { banner };
}

export const unavailable = {
  code: "unavailable",
  message: "The action could not finish. Try again.",
} as const;

export const conflict = {
  code: "conflict",
  message: "This action conflicts with the current workspace state.",
} as const;
