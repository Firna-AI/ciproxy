// The billing pages and their queries. Each page names its workspace by its
// opaque ID. The checkout page names the exact plan choice to review, and
// the return page the exact Checkout session that Stripe returned. IDs are
// opaque, so they pass through URLSearchParams unchanged.

import type { PlanSelection, PlanTier } from "../api/billing-types.ts";
import { idParam, workspaceHref } from "../routing/pages.ts";

export const billingPaths = {
  plans: "/plans",
  checkout: "/checkout",
  checkoutReturn: "/checkout-return",
} as const;

export type BillingPath = (typeof billingPaths)[keyof typeof billingPaths];

/** One query value: a missing, repeated, or empty value is none. */
type QueryValue = string | readonly string[] | undefined;

function withSearch(path: string, query: Readonly<Record<string, string>>) {
  return `${path}?${new URLSearchParams(query).toString()}`;
}

export function billingHref(workspaceId: string): string {
  return workspaceHref("billing", workspaceId);
}

export function plansHref(workspaceId: string): string {
  return withSearch(billingPaths.plans, { workspace: workspaceId });
}

/** The review of one exact plan choice. */
export function checkoutHref(
  workspaceId: string,
  selection: PlanSelection,
): string {
  return withSearch(billingPaths.checkout, {
    workspace: workspaceId,
    tier: selection.tier,
    start_blocks: String(selection.add_ons.start_blocks),
    machines: String(selection.add_ons.machines),
  });
}

/** The return page of one exact Checkout session. */
export function checkoutReturnHref(
  workspaceId: string,
  sessionId: string,
): string {
  return withSearch(billingPaths.checkoutReturn, {
    workspace: workspaceId,
    session_id: sessionId,
  });
}

/**
 * A Stripe return to a billing page without a workspace, sent on with the
 * workspace that this tab saved before it left for Stripe. It keeps every
 * other single value of the query, such as the exact Checkout session. With
 * a workspace in the query, or nothing saved, it gives null.
 */
export function stripeReturnHref(
  path: BillingPath | "/billing",
  query: Readonly<Record<string, QueryValue>>,
  saved: string | null,
): string | null {
  if (idParam(query.workspace) !== null || saved === null) return null;
  const kept: Record<string, string> = {};
  for (const [name, value] of Object.entries(query)) {
    if (name !== "workspace" && typeof value === "string") kept[name] = value;
  }
  return withSearch(path, { ...kept, workspace: saved });
}

const tiers: ReadonlySet<string> = new Set<PlanTier>([
  "free",
  "team",
  "business",
]);

function isTier(value: string): value is PlanTier {
  return tiers.has(value);
}

/**
 * A whole number of add-ons in decimal text, or null. The API checks the
 * range of each quantity.
 */
function quantity(value: QueryValue): number | null {
  if (typeof value !== "string" || !/^\d+$/.test(value)) return null;
  const number = Number(value);
  return Number.isSafeInteger(number) ? number : null;
}

/** The plan choice of a checkout query, or null when a value is missing. */
export function checkoutQuery(query: {
  tier?: QueryValue;
  start_blocks?: QueryValue;
  machines?: QueryValue;
}): PlanSelection | null {
  const tier = typeof query.tier === "string" ? query.tier : "";
  const blocks = quantity(query.start_blocks);
  const machines = quantity(query.machines);
  if (!isTier(tier) || blocks === null || machines === null) return null;
  return { tier, add_ons: { start_blocks: blocks, machines } };
}

/** The exact Checkout session of the return query, or null. */
export function sessionIdQuery(value: QueryValue): string | null {
  return idParam(value);
}
