// The approved MVP pricing (plan: Pricing). Every screen that shows a price, a
// limit, or a tier feature reads it from here.

import type { TierKey } from "./org.ts";

export type Tier = {
  key: TierKey;
  name: string;
  price: number;
  starts: number;
  nodes: number;
  history: string;
  addOns: boolean;
  /** What the tier adds to the tier below it. */
  adds: readonly string[];
};

export const tiers: Readonly<Record<TierKey, Tier>> = {
  free: {
    key: "free",
    name: "Free",
    price: 0,
    starts: 2_000,
    nodes: 1,
    history: "7 days",
    addOns: false,
    adds: ["7 days of job history"],
  },
  team: {
    key: "team",
    name: "Team",
    price: 49,
    starts: 25_000,
    nodes: 3,
    history: "30 days",
    addOns: true,
    adds: [
      "Sticky Docker disks",
      "Alerts by email and Slack",
      "30 days of job history",
      "Email support",
    ],
  },
  business: {
    key: "business",
    name: "Business",
    price: 299,
    starts: 150_000,
    nodes: 10,
    history: "90 days",
    addOns: true,
    adds: [
      "Several GitHub organizations",
      "Single sign-on",
      "Audit log",
      "Custom VM images",
      "90 days of job history",
      "Priority support",
    ],
  },
};

export const tierOrder: readonly TierKey[] = ["free", "team", "business"];

/** Included in every tier. */
export const everyTier: readonly string[] = [
  "Own Linux machines",
  "Boat overflow",
  "Transparent cache",
  "Warm runners",
  "Savings report",
];

/** Monthly add-ons on Team and Business. */
export const addOnPrices = {
  startsBlock: 10_000,
  startsPrice: 10,
  nodePrice: 20,
} as const;
