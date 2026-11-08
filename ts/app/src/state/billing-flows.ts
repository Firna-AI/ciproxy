// The billing flows of a workspace: a payment, the review before it, the
// return from Stripe Checkout, the billing portal, and Move to Free. Each
// flow that leaves for Stripe saves what this tab needs to come back: the
// workspace of the Stripe page, and the exact URL of a payment that the
// person can continue. Every server outcome replaces that URL, and no flow
// sends a payment request to continue a payment.

import { reviewRequest } from "../api/billing.ts";
import { submitPayment } from "../api/billing-payment.ts";
import type { PlanSelection, PlanStatus } from "../api/billing-types.ts";
import type { AppServices } from "../services/services.ts";
import { stripeReturnHref, type BillingPath } from "../view/billing-routes.ts";
import { createActionStore, type ActionStore } from "./billing-actions.ts";
import {
  clearPaymentContext,
  paymentContext,
  savePaymentContext,
  type PaymentContextKind,
} from "./billing-context.ts";
import {
  clearOpenPayment,
  openPayment,
  recordOutcome,
  type OpenPayment,
} from "./billing-open-payment.ts";
import type { PageSnapshot } from "./billing-page.ts";
import { createPaymentStore, type PaymentStore } from "./billing-payment.ts";
import { createReturnStore, type ReturnStore } from "./billing-return.ts";
import { createReviewStore, type ReviewStore } from "./billing-review.ts";
import {
  bannerStore,
  billingEndpoints,
  billingSettled,
  paymentGate,
  usageStore,
} from "./billing.ts";

export function createPagePayment(
  services: AppServices,
  workspaceId: string,
): PaymentStore {
  const { tab } = services.env;
  return createPaymentStore({
    send: async (submission) => {
      // A new or repeated payment request replaces any saved URL.
      clearOpenPayment(tab, workspaceId);
      const result = await submitPayment(services.api, workspaceId, submission);
      if (result.ok) recordOutcome(tab, workspaceId, result.value);
      return result;
    },
    mutate: services.session.mutate,
    canStart: () => paymentGate(services, workspaceId) === "open",
    leave: (url) => {
      savePaymentContext(tab, "checkout", workspaceId);
      services.navigator.leave(url);
    },
    settled: () => billingSettled(services, workspaceId),
  });
}

export function createReview(
  services: AppServices,
  workspaceId: string,
  initial: PlanSelection,
): ReviewStore {
  return createReviewStore({
    initial,
    read: (wanted) =>
      bannerStore(services, workspaceId).readPage(
        () =>
          billingEndpoints(services).review(workspaceId, reviewRequest(wanted)),
        (page) => page.banner,
      ),
    report: services.session.report,
  });
}

export function createCheckoutReturn(
  services: AppServices,
  workspaceId: string,
  sessionId: string,
): ReturnStore {
  const { tab } = services.env;
  const store: ReturnStore = createReturnStore({
    reconcile: () =>
      bannerStore(services, workspaceId).readPage(
        async () => {
          const result = await billingEndpoints(services).checkoutReturn(
            workspaceId,
            sessionId,
          );
          if (result.ok) recordOutcome(tab, workspaceId, result.value.outcome);
          return result;
        },
        (page) => page.banner,
      ),
    mutate: services.session.mutate,
    timers: services.env.timers,
    visibility: services.env.visibility,
    settled: () => {
      const current = store.state.get();
      const ended =
        current.status === "settled" &&
        (current.outcome.state === "confirmed" ||
          current.outcome.state === "unfinished");
      if (ended) clearPaymentContext(tab, "checkout");
      return billingSettled(services, workspaceId);
    },
  });
  return store;
}

/**
 * The payment that the page can offer to continue: the URL that this tab
 * saved, while the latest read succeeded, says that the payment of the
 * workspace still waits, and says that this person may pay. Continuing
 * opens that URL and sends no request.
 */
export function continuablePayment<T>(
  services: AppServices,
  workspaceId: string,
  snapshot: PageSnapshot<T>,
  status: (page: T) => PlanStatus,
): OpenPayment | null {
  if (snapshot.status !== "ready" || snapshot.refreshFailure !== null) {
    return null;
  }
  const current = status(snapshot.page);
  if (!current.payment_pending || !current.can_pay) return null;
  return openPayment(services.env.tab, workspaceId);
}

/** Open the Stripe billing portal: the invoices, the card, and Keep plan. */
export function createPortalAction(
  services: AppServices,
  workspaceId: string,
): ActionStore {
  return createActionStore({
    request: () => billingEndpoints(services).portal(workspaceId),
    mutate: services.session.mutate,
    done: (link) => {
      savePaymentContext(services.env.tab, "portal", workspaceId);
      services.navigator.leave(link.url);
    },
  });
}

/** Schedule Move to Free, then show Plan and usage with the end date. */
export function createMoveToFree(
  services: AppServices,
  workspaceId: string,
  then: () => void,
): ActionStore {
  return createActionStore({
    request: () => billingEndpoints(services).moveToFree(workspaceId),
    mutate: services.session.mutate,
    done: async () => {
      await billingSettled(services, workspaceId);
      then();
    },
  });
}

/**
 * Where a Stripe return without a workspace goes: the same route with the
 * workspace that this tab saved before it left for that Stripe page. Never
 * the workspace that the browser remembers, which other pages change.
 */
export function stripeReturnTarget(
  services: AppServices,
  kind: PaymentContextKind,
  path: BillingPath | "/billing",
  query: Readonly<Record<string, string | readonly string[] | undefined>>,
): string | null {
  return stripeReturnHref(path, query, paymentContext(services.env.tab, kind));
}

/**
 * Try again of Checkout after a failed Plan and usage read: read it again,
 * and the review too when that read failed. Nothing pays here.
 */
export async function checkoutReload(
  services: AppServices,
  workspaceId: string,
  review: ReviewStore,
): Promise<void> {
  const reviewFailed = review.state.get().review.status === "failed";
  await Promise.all([
    usageStore(services, workspaceId).load(),
    reviewFailed ? review.retry() : null,
  ]);
}
