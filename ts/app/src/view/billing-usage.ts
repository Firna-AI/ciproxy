// Plan and usage, from the page that the API gives: the plan, the use of the
// period against it, and the notices of this page. The API counts the use,
// the forecast, the day on which the pace passes the plan, and the offer;
// this model only words them.

import type { PlanStatus, PlanUsagePage } from "../api/billing-types.ts";
import type { OpenPayment } from "../state/billing-open-payment.ts";
import {
  capacityText,
  dayText,
  lastDayText,
  machinesText,
  monthlyText,
  selectionText,
  tierNames,
} from "./billing-format.ts";
import {
  dotJoin,
  formatCount,
  formatUtcDate,
  formatUtcRange,
  formatWhole,
} from "./exact-format.ts";

export type MeterModel = {
  label: string;
  /** "8,640 of 25,000". */
  value: string;
  /** The filled share of the bar, from 0 to 1. */
  ratio: number;
  warning: boolean;
  detail: string | null;
};

export type PlanModel = {
  name: string;
  paid: boolean;
  /** "$49 a month · renews on 4 December". */
  price: string;
  /** "25,000 starts a month and 3 own machines". */
  capacity: string;
};

/** The notices of Plan and usage; the banner of every page is apart. */
export type UsageNotice =
  | {
      kind: "confirming";
      title: string;
      body: string;
      /** The saved Stripe page of this payment, which the person continues. */
      resume: OpenPayment | null;
    }
  | { kind: "ends"; title: string; body: string; keep: string }
  | {
      kind: "pace";
      title: string;
      body: string;
      /** Plans for another tier, or the add-ons for the same tier. */
      action: "compare" | "add-ons";
    };

export type UsageModel = {
  plan: PlanModel;
  period: string;
  starts: MeterModel;
  machines: MeterModel;
  /** The organizations, only while they pass the plan. */
  organizations: { value: string; detail: string } | null;
  notices: readonly UsageNotice[];
};

function isZero(value: string): boolean {
  return /^0+(\.0+)?$/.test(value);
}

function ratio(used: string, limit: string): number {
  const max = Number(limit);
  if (!(max > 0)) return 1;
  return Math.min(Number(used) / max, 1);
}

function planModel(page: PlanUsagePage): PlanModel {
  const { status, current } = page;
  const paid = status.selection.tier !== "free";
  const when = !paid
    ? null
    : status.cancel_at
      ? `ends on ${lastDayText(status.cancel_at)}`
      : `renews on ${dayText(status.period_end)}`;
  const price = monthlyText(current.monthly_cents);
  return {
    name: tierNames[status.selection.tier],
    paid,
    price: when === null ? price : dotJoin(price, when),
    capacity: capacityText(current),
  };
}

function paceNotice(page: PlanUsagePage, workspace: string): UsageNotice[] {
  if (!page.crossing_day) return [];
  const { current, offer, status } = page;
  const tier = tierNames[status.selection.tier];
  return [
    {
      kind: "pace",
      title: `At this pace, ${workspace} passes the ${formatCount(current.included_starts)} starts of ${tier} on ${formatUtcDate(page.crossing_day)}`,
      body: `Jobs keep running past it. ${selectionText(offer.selection)} includes ${formatCount(offer.included_starts)} starts a month for ${monthlyText(offer.monthly_cents)}.`,
      action:
        offer.selection.tier === status.selection.tier ? "add-ons" : "compare",
    },
  ];
}

function endNotice(page: PlanUsagePage, workspace: string): UsageNotice[] {
  const { status } = page;
  if (status.selection.tier === "free" || !status.cancel_at) return [];
  const tier = tierNames[status.selection.tier];
  return [
    {
      kind: "ends",
      title: `${tier} ends on ${lastDayText(status.cancel_at)}`,
      body: `${workspace} moves to Free on ${dayText(status.cancel_at)}. Jobs keep running on all ${machinesText(page.machines)}.`,
      keep: `Keep ${tier}`,
    },
  ];
}

/**
 * The notice of a payment or an operation that the server waits for. A
 * payment that this tab can continue names its Stripe page: an open Checkout
 * or the page of an unpaid invoice. An unpaid invoice does not tell why
 * Stripe took no payment, so the notice names no bank.
 */
export type PendingNotice = Extract<UsageNotice, { kind: "confirming" }>;

export function pendingNotice(
  status: PlanStatus,
  resume: OpenPayment | null,
): PendingNotice | null {
  if (status.payment_pending && resume?.kind === "checkout") {
    return {
      kind: "confirming",
      title: "Your payment is not finished",
      body: "Stripe took no payment yet. Your plan changes when Stripe confirms the payment, and jobs keep running.",
      resume,
    };
  }
  if (status.payment_pending && resume?.kind === "bank") {
    return {
      kind: "confirming",
      title: "Your payment is not finished",
      body: "Finish it on its Stripe page. Your plan changes when Stripe confirms the payment, and jobs keep running.",
      resume,
    };
  }
  if (status.payment_pending) {
    return {
      kind: "confirming",
      title: "Stripe is confirming a payment",
      body: "Your plan changes when Stripe confirms it. This page updates by itself, and jobs keep running.",
      resume: null,
    };
  }
  if (status.operation_in_progress) {
    return {
      kind: "confirming",
      title: "A billing change is in progress",
      body: "This page updates by itself when it ends. Jobs keep running.",
      resume: null,
    };
  }
  return null;
}

function startsMeter(page: PlanUsagePage): MeterModel {
  const limit = page.current.included_starts;
  const over = !isZero(page.starts_over);
  return {
    label: "Starts",
    value: `${formatCount(page.starts)} of ${formatCount(limit)}`,
    ratio: ratio(page.starts, limit),
    warning: over || page.crossing_day !== null,
    detail: over
      ? `${formatWhole(page.starts_over)} over. Jobs keep running.`
      : `About ${formatWhole(page.expected_starts)} by the end of the period at this pace.`,
  };
}

function machinesMeter(page: PlanUsagePage): MeterModel {
  const limit = page.current.included_machines;
  const over = !isZero(page.machines_over);
  return {
    label: "Own machines",
    value: `${formatCount(page.machines)} of ${formatCount(limit)}`,
    ratio: ratio(page.machines, limit),
    warning: over,
    detail: over
      ? `${formatCount(page.machines_over)} over. Every machine runs jobs.`
      : null,
  };
}

export function usageModel(
  page: PlanUsagePage,
  workspace: string,
  resume: OpenPayment | null = null,
): UsageModel {
  const { status } = page;
  const pending = pendingNotice(status, resume);
  return {
    plan: planModel(page),
    period: formatUtcRange(status.period_start, status.period_end),
    starts: startsMeter(page),
    machines: machinesMeter(page),
    organizations: isZero(page.organizations_over)
      ? null
      : {
          value: formatCount(page.organizations),
          detail: `${formatCount(page.organizations_over)} over. Jobs keep running in every active organization.`,
        },
    notices: [
      ...(pending === null ? [] : [pending]),
      ...endNotice(page, workspace),
      ...paceNotice(page, workspace),
    ],
  };
}
