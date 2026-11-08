// The billing stores of one workspace, built from the services of the
// document: Plan and usage, Plans, and the banner of each scope, one of each
// for each workspace. A read that shows no pending payment ends the open
// payment that this tab saved for the workspace. billing-flows.ts builds the
// stores of the pages that pay, review, or return.

import { createBannerEndpoint, type BannerScope } from "../api/banner.ts";
import {
  createBillingEndpoints,
  type BillingEndpoints,
} from "../api/billing.ts";
import type { ApiResult } from "../api/client.ts";
import type {
  PlanStatus,
  PlanUsagePage,
  PlansPage,
} from "../api/billing-types.ts";
import type { AppServices } from "../services/services.ts";
import { createBannerStore, type BannerStore } from "./billing-banner.ts";
import { ownedCache } from "./billing-cache.ts";
import { clearOpenPayment } from "./billing-open-payment.ts";
import {
  createPageStore,
  statusWaits,
  type PageStore,
} from "./billing-page.ts";

const endpointCache = ownedCache<BillingEndpoints>();
const usageCache = ownedCache<PageStore<PlanUsagePage>>();
const plansCache = ownedCache<PageStore<PlansPage>>();
const bannerCache = ownedCache<BannerStore>();

export function billingEndpoints(services: AppServices): BillingEndpoints {
  return endpointCache(services, "", () =>
    createBillingEndpoints(services.api),
  );
}

/** Pass a read through, and end the saved open payment when none waits. */
function forgetEnded<T>(
  services: AppServices,
  workspaceId: string,
  status: (page: T) => PlanStatus,
) {
  return (result: ApiResult<T>): ApiResult<T> => {
    if (result.ok && !status(result.value).payment_pending) {
      clearOpenPayment(services.env.tab, workspaceId);
    }
    return result;
  };
}

export function usageStore(
  services: AppServices,
  workspaceId: string,
): PageStore<PlanUsagePage> {
  return usageCache(services, workspaceId, () =>
    createPageStore({
      read: () =>
        billingEndpoints(services)
          .usage(workspaceId)
          .then(forgetEnded(services, workspaceId, (page) => page.status)),
      report: services.session.report,
      waits: (page) => statusWaits(page.status),
      timers: services.env.timers,
      visibility: services.env.visibility,
    }),
  );
}

export function plansStore(
  services: AppServices,
  workspaceId: string,
): PageStore<PlansPage> {
  return plansCache(services, workspaceId, () =>
    createPageStore({
      read: () =>
        billingEndpoints(services)
          .plans(workspaceId)
          .then(
            forgetEnded(services, workspaceId, (page) => page.usage.status),
          ),
      report: services.session.report,
      waits: (page) => statusWaits(page.usage.status),
      timers: services.env.timers,
      visibility: services.env.visibility,
    }),
  );
}

export function bannerStore(
  services: AppServices,
  workspaceId: string,
  scope: BannerScope = "page",
): BannerStore {
  return bannerCache(services, `${scope}:${workspaceId}`, () =>
    createBannerStore({
      read: () => createBannerEndpoint(services.api).banner(workspaceId, scope),
      report: services.session.report,
      now: services.env.now,
    }),
  );
}

/** Read every shown billing value of the workspace again after a change. */
export async function billingSettled(
  services: AppServices,
  workspaceId: string,
): Promise<void> {
  const plans = plansStore(services, workspaceId);
  await Promise.all([
    usageStore(services, workspaceId).load(),
    plans.state.get().status === "loading" ? null : plans.load(),
    bannerStore(services, workspaceId).refresh(),
  ]);
}

/**
 * Whether a new payment may start: the latest read of Plan and usage
 * succeeded and shows no payment or operation that the server waits for. A
 * failed latest read leaves the state unknown, so no payment starts from the
 * page before it.
 */
export function paymentMayStart(
  services: AppServices,
  workspaceId: string,
): boolean {
  const usage = usageStore(services, workspaceId).state.get();
  return (
    usage.status === "ready" &&
    usage.refreshFailure === null &&
    usage.page.status.can_pay &&
    !statusWaits(usage.page.status)
  );
}
