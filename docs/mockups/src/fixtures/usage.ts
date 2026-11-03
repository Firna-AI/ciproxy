// Plan and usage of mokly-ai at the moments the billing screens show. Team
// started on 4 November, so a Team period runs from the 4th to the 3rd.

import type { TierKey } from "./org.ts";

export type UsageScene =
  | "steady"
  | "near-limit"
  | "over-starts"
  | "over-nodes"
  | "free"
  | "free-over"
  | "payment-failed"
  | "moved-to-free";

export type Usage = {
  tier: TierKey;
  period: string;
  starts: number;
  /** Starts by the end of the period at the current pace. */
  projected: number;
  /** The day the pace passes the included starts, when it does. */
  passesOn?: string;
  nodes: number;
  extraStarts: number;
  extraNodes: number;
  renews?: string;
};

const team = {
  tier: "team",
  period: "4 November – 3 December",
  extraStarts: 0,
  extraNodes: 0,
  renews: "4 December",
} as const;

export const usageScenes: Readonly<Record<UsageScene, Usage>> = {
  steady: { ...team, starts: 8_640, projected: 18_500, nodes: 3 },
  "near-limit": {
    ...team,
    starts: 21_640,
    projected: 25_900,
    passesOn: "2 December",
    nodes: 3,
  },
  "over-starts": {
    ...team,
    starts: 25_310,
    projected: 26_100,
    passesOn: "2 December",
    nodes: 3,
  },
  "over-nodes": { ...team, starts: 8_640, projected: 18_500, nodes: 4 },
  free: {
    tier: "free",
    period: "1 – 30 November",
    starts: 1_240,
    projected: 18_200,
    passesOn: "3 November",
    nodes: 1,
    extraStarts: 0,
    extraNodes: 0,
  },
  "free-over": {
    tier: "free",
    period: "1 – 30 November",
    starts: 2_214,
    projected: 18_200,
    passesOn: "4 November",
    nodes: 1,
    extraStarts: 0,
    extraNodes: 0,
  },
  "payment-failed": {
    ...team,
    period: "4 December – 3 January",
    starts: 1_150,
    projected: 18_300,
    nodes: 3,
    renews: "4 January",
  },
  "moved-to-free": {
    tier: "free",
    period: "1 – 31 December",
    starts: 11_020,
    projected: 18_900,
    passesOn: "4 December",
    nodes: 3,
    extraStarts: 0,
    extraNodes: 0,
  },
};

/** Dates of the payment problems. */
export const paymentDates = {
  failed: "4 December",
  retry: "7 December",
  ended: "18 December",
} as const;
