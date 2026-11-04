// The feature comparison of the pricing page, by area. The limits, prices,
// and feature names come from tiers.ts; a feature that a tier adds is in every
// tier above it too.

import { formatCount, formatMoney } from "./format.ts";
import type { TierKey } from "./org.ts";
import { addOnPrices, feature, tierOrder, tiers } from "./tiers.ts";

/** A cell: included, not included, or a value such as "30 days". */
export type CompareValue = boolean | string;

export type CompareRow = {
  label: string;
  values: Readonly<Record<TierKey, CompareValue>>;
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

/** A feature that starts at one tier and stays in every tier above it. */
function from(start: TierKey, label: string): CompareRow {
  const first = tierOrder.indexOf(start);
  return each(label, (key) => tierOrder.indexOf(key) >= first);
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
      from("free", feature.ownMachines),
      from("free", feature.boat),
      from("free", feature.warmRunners),
      from("free", feature.cache),
      from("team", feature.stickyDisks),
      from("business", feature.images),
    ],
  },
  {
    title: "Dashboard",
    rows: [
      from("free", feature.savings),
      each("Job history", (key) => tiers[key].history),
      from("team", feature.alerts),
    ],
  },
  {
    title: "Organization",
    rows: [
      from("business", feature.organizations),
      from("business", feature.singleSignOn),
      from("business", feature.auditLog),
    ],
  },
  {
    title: "Support",
    rows: [
      from("team", feature.emailSupport),
      from("business", feature.prioritySupport),
    ],
  },
];
