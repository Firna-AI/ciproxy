import { randomUUID } from "node:crypto";

import type { components } from "../../api/index.ts";
import { fixtureTime, workspaceId } from "./fixtures-core.ts";
import { planOffer, planTiers } from "./fixtures-billing.ts";

export const fixtureCheckoutId = "fixture-checkout-m21";
export const paymentStates = [
  "open",
  "confirming",
  "confirmed",
  "unfinished",
  "payment_required",
] as const;
export type PaymentState = components["schemas"]["PaymentOutcome"]["state"];

function object(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function quantity(value: unknown): value is number {
  return typeof value === "number" && Number.isInteger(value) && value >= 0;
}

export function parseAddOns(
  value: unknown,
): components["schemas"]["PlanAddOns"] | undefined {
  if (
    !object(value) ||
    !quantity(value.start_blocks) ||
    !quantity(value.machines)
  )
    return undefined;
  return { start_blocks: value.start_blocks, machines: value.machines };
}

export function parseSelection(
  value: unknown,
): components["schemas"]["PlanSelection"] | undefined {
  if (!object(value)) return undefined;
  const tier = planTiers.find((item) => item === value.tier);
  const addOns = parseAddOns(value.add_ons);
  return tier === undefined || addOns === undefined
    ? undefined
    : { tier, add_ons: addOns };
}

export function parsePaidChange(
  value: unknown,
): components["schemas"]["PaidChangePayment"] | undefined {
  return object(value) && typeof value.quote_id === "string"
    ? { quote_id: value.quote_id }
    : undefined;
}

export function reviewSelection(
  query: URLSearchParams,
  suggested: components["schemas"]["PlanSelection"],
): components["schemas"]["PlanSelection"] | undefined {
  const tier = query.has("tier")
    ? planTiers.find((item) => item === query.get("tier"))
    : suggested.tier;
  if (tier === undefined) return undefined;
  const defaults =
    tier === suggested.tier
      ? suggested.add_ons
      : { start_blocks: 0, machines: 0 };
  const parse = (key: string, fallback: number): number => {
    const value = query.get(key);
    return value === null ? fallback : /^\d+$/.test(value) ? Number(value) : -1;
  };
  const addOns = parseAddOns({
    start_blocks: parse("start_blocks", defaults.start_blocks),
    machines: parse("machines", defaults.machines),
  });
  return addOns === undefined ? undefined : { tier, add_ons: addOns };
}

export function checkoutReview(
  selection: components["schemas"]["PlanSelection"],
  usage: components["schemas"]["PlanUsagePage"],
): components["schemas"]["CheckoutReviewPage"] {
  const choice = planOffer(selection);
  const prorated = usage.status.selection.tier !== "free";
  const remaining = BigInt(
    Date.parse(usage.status.period_end) - Date.parse(fixtureTime),
  );
  const period = BigInt(
    Date.parse(usage.status.period_end) - Date.parse(usage.status.period_start),
  );
  const difference =
    BigInt(choice.monthly_cents) - BigInt(usage.current.monthly_cents);
  const quote: components["schemas"]["PaidQuoteReference"] | null = prorated
    ? {
        id: randomUUID(),
        expires_at: new Date(
          Math.min(
            Date.parse(fixtureTime) + 30 * 60 * 1000,
            Date.parse(usage.status.period_end),
          ),
        ).toISOString(),
      }
    : null;
  return {
    choice,
    billing_day: prorated
      ? new Date(usage.status.period_start).getUTCDate()
      : 6,
    period_end: usage.status.period_end,
    prorated,
    quote,
    today_cents: prorated
      ? difference <= 0n
        ? "0"
        : String((difference * remaining + period / 2n) / period)
      : choice.monthly_cents,
    banner: usage.banner,
  } satisfies components["schemas"]["CheckoutReviewPage"];
}

export function paymentOutcome(
  state: PaymentState,
): components["schemas"]["PaymentOutcome"] {
  if (state === "open")
    return {
      state,
      url: `/__fixture/stripe/checkout?workspace=${workspaceId}&session_id=${fixtureCheckoutId}`,
    };
  if (state === "payment_required")
    return {
      state,
      payment_url: `/__fixture/stripe/payment?workspace=${workspaceId}`,
    };
  return { state };
}

export function parseCheckoutReturn(
  value: unknown,
): components["schemas"]["CheckoutReturn"] | undefined {
  return object(value) && typeof value.session_id === "string"
    ? { session_id: value.session_id }
    : undefined;
}
