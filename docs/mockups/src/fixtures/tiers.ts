// The approved MVP pricing (plan: Pricing) and the features of each tier.
// Every screen that shows a price, a limit, or a tier feature reads it from
// here. A feature starts at one tier and stays in every tier above it. A
// feature that does not ship at launch shows "Coming later" on every page
// that lists it. A workspace on Free holds one GitHub organization; Team and
// Business hold several, as the answer to item 48 says.

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
  /** The GitHub organizations that a workspace holds, when the tier limits it. */
  organizations?: number;
};

export const tiers: Readonly<Record<TierKey, Tier>> = {
  free: {
    key: "free",
    name: "Free",
    price: 0,
    starts: 2_000,
    machines: 1,
    history: "7 days",
    addOns: false,
    organizations: 1,
  },
  team: {
    key: "team",
    name: "Team",
    price: 49,
    starts: 25_000,
    machines: 3,
    history: "30 days",
    addOns: true,
  },
  business: {
    key: "business",
    name: "Business",
    price: 299,
    starts: 150_000,
    machines: 10,
    history: "90 days",
    addOns: true,
  },
};

export const tierOrder: readonly TierKey[] = ["free", "team", "business"];

export type FeatureKey =
  | "ownMachines"
  | "boat"
  | "cache"
  | "warmRunners"
  | "savings"
  | "stickyDisks"
  | "alerts"
  | "emailSupport"
  | "organizations"
  | "singleSignOn"
  | "auditLog"
  | "images"
  | "prioritySupport";

/** One tier feature: its name, the first tier that has it, and its launch. */
export type TierFeature = {
  name: string;
  /** The first tier that includes the feature. Every tier above has it too. */
  from: TierKey;
  /** The feature ships at launch. Otherwise each page says "Coming later". */
  launch: boolean;
};

/** One record for each feature, in the order of the tier lists. */
export const features: Readonly<Record<FeatureKey, TierFeature>> = {
  ownMachines: { name: "Own Linux machines", from: "free", launch: true },
  boat: { name: "Boat overflow", from: "free", launch: true },
  cache: { name: "Transparent cache", from: "free", launch: true },
  warmRunners: { name: "Warm runners", from: "free", launch: true },
  savings: { name: "Savings report", from: "free", launch: true },
  organizations: {
    name: "Several GitHub organizations",
    from: "team",
    launch: true,
  },
  stickyDisks: { name: "Sticky Docker disks", from: "team", launch: true },
  alerts: { name: "Alerts by email and Slack", from: "team", launch: false },
  emailSupport: { name: "Email support", from: "team", launch: true },
  singleSignOn: { name: "Single sign-on", from: "business", launch: false },
  auditLog: { name: "Audit log", from: "business", launch: false },
  images: { name: "Custom VM images", from: "business", launch: false },
  prioritySupport: { name: "Priority support", from: "business", launch: true },
};

/** The tier includes the feature: it starts at this tier or below. */
export function includes(tier: TierKey, feature: TierFeature): boolean {
  return tierOrder.indexOf(tier) >= tierOrder.indexOf(feature.from);
}

/** One line of a tier list. A feature that ships later says so. */
export type TierListItem = { name: string; later: boolean };

/**
 * Free lists all that it includes, and each tier above lists what it adds:
 * the features that ship at launch, the job history, then the features that
 * come later.
 */
export function tierFeatures(tier: Tier): {
  lead: string;
  items: readonly TierListItem[];
} {
  const added = Object.values(features).filter(
    (feature) => feature.from === tier.key,
  );
  const items: TierListItem[] = [
    ...added
      .filter((feature) => feature.launch)
      .map((feature) => ({ name: feature.name, later: false })),
    { name: `${tier.history} of job history`, later: false },
    ...added
      .filter((feature) => !feature.launch)
      .map((feature) => ({ name: feature.name, later: true })),
  ];
  const below = tierOrder[tierOrder.indexOf(tier.key) - 1];
  return {
    lead: below ? `Everything in ${tiers[below].name}, plus` : "Free includes",
    items,
  };
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
