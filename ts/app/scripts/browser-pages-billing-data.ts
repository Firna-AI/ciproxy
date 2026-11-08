import { createBillingEndpoints, reviewRequest } from "../src/api/billing.ts";
import type { PlanSelection, PlanUsagePage } from "../src/api/billing-types.ts";
import { createApiClient } from "../src/api/client.ts";
import { createBuildGate } from "../src/state/build-gate.ts";
import {
  billingHref,
  checkoutHref,
  checkoutReturnHref,
  plansHref,
} from "../src/view/billing-routes.ts";
import { workspaceId } from "./fixtures-core.ts";

export function billingPageUrls(selection: PlanSelection, sessionId: string) {
  return {
    usage: billingHref(workspaceId),
    plans: plansHref(workspaceId),
    review: checkoutHref(workspaceId, selection),
    returned: checkoutReturnHref(workspaceId, sessionId),
  };
}

export async function fixtureUsage(origin: string): Promise<PlanUsagePage> {
  const result = await fixtureBilling(origin).usage(workspaceId);
  if (!result.ok)
    throw new Error("The authoritative fixture usage is unavailable.");
  return result.value;
}

function fixtureBilling(origin: string) {
  const client = createApiClient({
    csrf: () => null,
    gate: createBuildGate(),
    transport: (path, init) => fetch(origin + path, init),
  });
  return createBillingEndpoints(client);
}

export async function fixtureReview(origin: string, selection: PlanSelection) {
  const result = await fixtureBilling(origin).review(
    workspaceId,
    reviewRequest(selection),
  );
  if (!result.ok) throw new Error("The authoritative review is unavailable.");
  return result.value;
}
