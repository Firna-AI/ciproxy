// The approved MVP pricing (plan: Pricing). Every screen that shows a price, a
// limit, or a tier feature reads it from here.

import { formatCount, unbroken } from "./format.ts";
import type { TierKey } from "./org.ts";

export type Tier = {
  key: TierKey;
  name: string;
  price: number;
  starts: number;
  /** The own machines that the tier includes. */
  machines: number;
  history: string;
  addOns: boolean;
  /** What the tier adds to the tier below it. */
  adds: readonly string[];
};

/** The names of the tier features, shared by the tier cards and the comparison. */
export const feature = {
  ownMachines: "Own Linux machines",
  boat: "Boat overflow",
  cache: "Transparent cache",
  warmRunners: "Warm runners",
  savings: "Savings report",
  stickyDisks: "Sticky Docker disks",
  alerts: "Alerts by email and Slack",
  emailSupport: "Email support",
  organizations: "Several GitHub organizations",
  singleSignOn: "Single sign-on",
  auditLog: "Audit log",
  images: "Custom VM images",
  prioritySupport: "Priority support",
} as const;

export const tiers: Readonly<Record<TierKey, Tier>> = {
  free: {
    key: "free",
    name: "Free",
    price: 0,
    starts: 2_000,
    machines: 1,
    history: "7 days",
    addOns: false,
    adds: ["7 days of job history"],
  },
  team: {
    key: "team",
    name: "Team",
    price: 49,
    starts: 25_000,
    machines: 3,
    history: "30 days",
    addOns: true,
    adds: [
      feature.stickyDisks,
      feature.alerts,
      "30 days of job history",
      feature.emailSupport,
    ],
  },
  business: {
    key: "business",
    name: "Business",
    price: 299,
    starts: 150_000,
    machines: 10,
    history: "90 days",
    addOns: true,
    adds: [
      feature.organizations,
      feature.singleSignOn,
      feature.auditLog,
      feature.images,
      "90 days of job history",
      feature.prioritySupport,
    ],
  },
};

export const tierOrder: readonly TierKey[] = ["free", "team", "business"];

/** Included in every tier. */
export const everyTier: readonly string[] = [
  feature.ownMachines,
  feature.boat,
  feature.cache,
  feature.warmRunners,
  feature.savings,
];

/** Free lists all it includes; each tier above lists what it adds. */
export function tierFeatures(tier: Tier): {
  lead: string;
  items: readonly string[];
} {
  if (tier.key === "free")
    return { lead: "Free includes", items: [...everyTier, ...tier.adds] };
  const below = tier.key === "team" ? "Free" : "Team";
  return { lead: `Everything in ${below}, plus`, items: tier.adds };
}

/** Monthly add-ons on Team and Business. */
export const addOnPrices = {
  startsBlock: 10_000,
  startsPrice: 10,
  machinePrice: 20,
} as const;

/** "1 own machine", "3 own machines", whole on one line. */
export function ownMachines(count: number): string {
  return unbroken(`${count} own ${count === 1 ? "machine" : "machines"}`);
}

/** "25,000 starts", whole on one line. */
export function startsText(count: number): string {
  return unbroken(`${formatCount(count)} starts`);
}

/** What Free includes, in one line, for the public site. */
export const freePlanNote = `Free for ${tiers.free.machines === 1 ? "one machine" : `${tiers.free.machines} machines`} and ${formatCount(tiers.free.starts)} starts a month.`;
