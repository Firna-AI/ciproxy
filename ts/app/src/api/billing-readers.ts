// The readers of the small billing values that several pages share: a plan
// choice, its price and capacity, the confirmed status, and the outcome of a
// payment. Each reader throws a ParseError on a wrong shape, as parse-core
// says, so a page reader rejects the whole response.

import type {
  PaymentOutcome,
  PlanAddOns,
  PlanOffer,
  PlanSelection,
  PlanStatus,
  PlanTier,
  SubscriptionStatus,
} from "./billing-types.ts";
import {
  boolean,
  decimal,
  integer,
  object,
  oneOf,
  optional,
  ParseError,
  string,
  time,
} from "./parse-core.ts";

export const readPlanTier = oneOf<PlanTier>({
  free: true,
  team: true,
  business: true,
});

const readSubscription = oneOf<SubscriptionStatus>({
  none: true,
  active: true,
  payment_failed: true,
  ended: true,
});

export function readAddOns(value: unknown): PlanAddOns {
  const fields = object(value, "add-ons");
  return {
    start_blocks: integer(fields.start_blocks),
    machines: integer(fields.machines),
  };
}

export function readSelection(value: unknown): PlanSelection {
  const fields = object(value, "plan choice");
  return {
    tier: readPlanTier(fields.tier),
    add_ons: readAddOns(fields.add_ons),
  };
}

export function readPlanOffer(value: unknown): PlanOffer {
  const fields = object(value, "plan offer");
  return {
    selection: readSelection(fields.selection),
    included_starts: decimal(fields.included_starts),
    included_machines: decimal(fields.included_machines),
    monthly_cents: decimal(fields.monthly_cents),
  };
}

export function readPlanStatus(value: unknown): PlanStatus {
  const fields = object(value, "plan status");
  return {
    selection: readSelection(fields.selection),
    subscription: readSubscription(fields.subscription),
    period_start: time(fields.period_start),
    period_end: time(fields.period_end),
    cancel_at: optional(fields.cancel_at, time),
    payment_pending: boolean(fields.payment_pending),
    operation_in_progress: boolean(fields.operation_in_progress),
    can_pay: boolean(fields.can_pay),
  };
}

export function readPaymentOutcome(value: unknown): PaymentOutcome {
  const fields = object(value, "payment outcome");
  switch (fields.state) {
    case "open":
      return { state: "open", url: string(fields.url) };
    case "confirming":
      return { state: "confirming" };
    case "confirmed":
      return { state: "confirmed" };
    case "unfinished":
      return { state: "unfinished" };
    case "payment_required":
      return {
        state: "payment_required",
        payment_url: optional(fields.payment_url, string),
      };
    default:
      throw new ParseError("payment state");
  }
}
