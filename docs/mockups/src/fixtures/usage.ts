// Plan and usage of mokly-ai at the moments the billing screens show. Each
// tier counts only the starts inside its own period, and a period begins on
// the day its tier begins: Free from sign-up on 1 September, Team from 4
// September, so a Team period runs from the 4th to the 3rd, and Free again
// from 18 December, when Stripe ended Team after failed payments. A move
// between paid tiers keeps the period, and a move to Free waits for its end.

import { dateRange, unbroken } from "./format.ts";
import type { TierKey } from "./org.ts";

export type UsageScene =
  | "steady"
  | "near-limit"
  | "over-starts"
  | "over-machines"
  | "free"
  | "free-over"
  | "payment-failed"
  | "payment-failed-over-limit"
  | "moved-to-free"
  | "free-scheduled";

export type Usage = {
  tier: TierKey;
  period: string;
  starts: number;
  /** Starts by the end of the period at the current pace. */
  projected: number;
  /** The day the pace passes the included starts, when it does. */
  passesOn?: string;
  /** Own machines connected now. */
  machines: number;
  extraStarts: number;
  extraMachines: number;
  renews?: string;
  /** The last day of a paid tier after a move to Free. */
  ends?: string;
};

/** The last day of the Team period of November. */
export const teamPeriodEnd = unbroken("3 December");

const team = {
  tier: "team",
  period: dateRange("4 November", teamPeriodEnd),
  extraStarts: 0,
  extraMachines: 0,
  renews: unbroken("4 December"),
} as const;

const free = {
  tier: "free",
  period: dateRange("1 September", "30 September"),
  projected: 18_200,
  machines: 1,
  extraStarts: 0,
  extraMachines: 0,
} as const;

/** Stripe could not charge the card for the December period. */
const paymentFailed: Usage = {
  ...team,
  period: dateRange("4 December", "3 January"),
  starts: 1_150,
  projected: 18_300,
  machines: 3,
  renews: unbroken("4 January"),
};

export const usageScenes: Readonly<Record<UsageScene, Usage>> = {
  steady: { ...team, starts: 8_640, projected: 18_500, machines: 3 },
  "near-limit": {
    ...team,
    starts: 21_640,
    projected: 25_900,
    passesOn: unbroken("2 December"),
    machines: 3,
  },
  "over-starts": { ...team, starts: 25_310, projected: 26_100, machines: 3 },
  "over-machines": { ...team, starts: 8_640, projected: 18_500, machines: 4 },
  free: { ...free, starts: 1_240, passesOn: unbroken("3 September") },
  "free-over": { ...free, starts: 2_214, passesOn: unbroken("4 September") },
  "payment-failed": paymentFailed,
  /** The payment failed while office-mini-1 runs as a fourth machine. */
  "payment-failed-over-limit": { ...paymentFailed, machines: 4 },
  "moved-to-free": {
    ...free,
    period: dateRange("18 December", "17 January"),
    starts: 420,
    projected: 13_400,
    passesOn: unbroken("23 December"),
    machines: 3,
  },
  "free-scheduled": {
    ...team,
    starts: 8_640,
    projected: 18_500,
    machines: 3,
    renews: undefined,
    ends: teamPeriodEnd,
  },
};

/** The first day of Free after the move to Free of the November period. */
export const freeFrom = unbroken("4 December");

/** The last day of the Free period of September, for the pace of the use. */
export const freePeriodEnd = unbroken("30 September");

/** The first Team period, which begins when the checkout confirms. */
export const firstTeamPeriod = {
  period: dateRange("4 September", "3 October"),
  starts: 0,
} as const;

/** Dates of the payment problems. */
export const paymentDates = {
  failed: unbroken("4 December"),
  retry: unbroken("7 December"),
  ended: unbroken("18 December"),
} as const;

/**
 * A move from Team to Business on 18 November: Stripe charges the price
 * difference for the 16 days left of the 30-day period.
 */
export const businessMove = {
  daysLeft: 16,
  periodDays: 30,
  starts: usageScenes.steady.starts,
} as const;

/** The prorated charge of a move between two monthly prices, in dollars. */
export function proratedCharge(from: number, to: number): number {
  const share = businessMove.daysLeft / businessMove.periodDays;
  return Math.round((to - from) * share * 100) / 100;
}
