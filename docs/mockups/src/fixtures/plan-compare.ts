// The feature comparison of the pricing page, by area. The limits, prices,
// and features come from tiers.ts: a feature that a tier adds is in every
// tier above it too, and a feature that comes later says so in its row.

import { formatCount, formatMoney } from "./format.ts";
import type { TierKey } from "./org.ts";
import {
  addOnPrices,
  features,
  includes,
  tiers,
  type FeatureKey,
} from "./tiers.ts";

/** A cell: included, not included, or a value such as "30 days". */
export type CompareValue = boolean | string;

export type CompareRow = {
  label: string;
  values: Readonly<Record<TierKey, CompareValue>>;
  /** The feature comes later, so an included cell says "Coming later". */
  later?: boolean;
};

export type CompareGroup = { title: string; rows: readonly CompareRow[] };

/** A row whose value comes from each tier. */
function each(
  label: string,
  value: (key: TierKey) => CompareValue,
): CompareRow {
  return {
    label,
    values: {
      free: value("free"),
      team: value("team"),
      business: value("business"),
    },
  };
}

/** The row of a tier feature, from its record. */
function featureRow(key: FeatureKey): CompareRow {
  const feature = features[key];
  return {
    ...each(feature.name, (tier) => includes(tier, feature)),
    later: !feature.launch,
  };
}

/** A monthly add-on price on the tiers that offer add-ons. */
function addOn(label: string, price: number): CompareRow {
  return each(label, (key) => (tiers[key].addOns ? formatMoney(price) : false));
}

export const compareGroups: readonly CompareGroup[] = [
  {
    title: "Usage",
    rows: [
      each("Starts a month", (key) => formatCount(tiers[key].starts)),
      each("Own machines", (key) => formatCount(tiers[key].machines)),
      addOn(
        `Extra ${formatCount(addOnPrices.startsBlock)} starts, per month`,
        addOnPrices.startsPrice,
      ),
      addOn("Extra own machine, per month", addOnPrices.machinePrice),
    ],
  },
  {
    title: "Runners",
    rows: [
      featureRow("ownMachines"),
      featureRow("boat"),
      featureRow("warmRunners"),
      featureRow("cache"),
      featureRow("stickyDisks"),
      featureRow("images"),
    ],
  },
  {
    title: "Dashboard",
    rows: [
      featureRow("savings"),
      each("Job history", (key) => tiers[key].history),
      featureRow("alerts"),
    ],
  },
  {
    title: "Organization",
    rows: [
      featureRow("organizations"),
      featureRow("singleSignOn"),
      featureRow("auditLog"),
    ],
  },
  {
    title: "Support",
    rows: [featureRow("emailSupport"), featureRow("prioritySupport")],
  },
];
