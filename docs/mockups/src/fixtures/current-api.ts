// The billing pages as the current API gives them, apart from the known
// scenes. The API sends one offer with each use: the plan and add-ons that
// cover it. It words a banner without the plan or the limit that passed, and
// it prices only the saved add-ons and the offer. Each view reuses a usage
// scene of usage.ts, so its numbers stay those of the story.

import { formatCount, monthly } from "./format.ts";
import type { TierKey } from "./org.ts";
import { addOnPrices, ownMachines, tiers } from "./tiers.ts";
import { usageScenes, type UsageScene } from "./usage.ts";

/** Add-on quantities: blocks of starts and extra own machines. */
export type AddOnCount = { starts: number; machines: number };

/** A paid choice: a tier with its add-on quantities. */
export type ApiOffer = { tier: TierKey; addOns: AddOnCount };

const noAddOns: AddOnCount = { starts: 0, machines: 0 };

/** The add-ons that cover a use on its own tier, where the plan does not. */
const coveringAddOns: Partial<Record<UsageScene, AddOnCount>> = {
  "near-limit": { starts: 1, machines: 0 },
  "over-starts": { starts: 1, machines: 0 },
  "over-machines": { starts: 0, machines: 1 },
};

/**
 * The offer of a scene: Team on Free, the covering add-ons on a paid tier,
 * or the plan itself when it covers the use.
 */
export function apiOffer(scene: UsageScene): ApiOffer {
  const { tier } = usageScenes[scene];
  if (tier === "free") return { tier: "team", addOns: noAddOns };
  return { tier, addOns: coveringAddOns[scene] ?? noAddOns };
}

/**
 * The scenes whose use passes a limit of the plan, so the API sends the
 * banner of a passed limit, unless a payment banner takes its place.
 */
export const passedLimit: ReadonlySet<UsageScene> = new Set<UsageScene>([
  "over-starts",
  "over-machines",
  "free-over",
  "free-organizations",
  "inactive-organization",
]);

export function sameCount(a: AddOnCount, b: AddOnCount): boolean {
  return a.starts === b.starts && a.machines === b.machines;
}

/** The monthly price of a choice, in dollars, from the published prices. */
export function choicePrice(offer: ApiOffer): number {
  const { starts, machines } = offer.addOns;
  return (
    tiers[offer.tier].price +
    starts * addOnPrices.startsPrice +
    machines * addOnPrices.machinePrice
  );
}

/** The starts a month that a choice includes. */
export function choiceStarts(offer: ApiOffer): number {
  return (
    tiers[offer.tier].starts + offer.addOns.starts * addOnPrices.startsBlock
  );
}

/** "35,000 starts a month and 3 own machines": what a choice includes. */
export function choiceCapacity(offer: ApiOffer): string {
  const machines = tiers[offer.tier].machines + offer.addOns.machines;
  return `${formatCount(choiceStarts(offer))} starts a month and ${ownMachines(machines)}`;
}

/** "Team with 1 block of starts", as the dashboard names a choice. */
export function choiceName(offer: ApiOffer): string {
  const { starts, machines } = offer.addOns;
  const parts = [
    ...(starts > 0
      ? [`${starts} ${starts === 1 ? "block" : "blocks"} of starts`]
      : []),
    ...(machines > 0
      ? [`${machines} extra own ${machines === 1 ? "machine" : "machines"}`]
      : []),
  ];
  const name = tiers[offer.tier].name;
  return parts.length === 0 ? name : `${name} with ${parts.join(" and ")}`;
}

/** "$59 a month", the monthly price of a choice. */
export function choiceMonthly(offer: ApiOffer): string {
  return monthly(choicePrice(offer));
}

/** The captions of the add-on quantities, which carry no price. */
export const apiAddOnCaptions = {
  starts: "Blocks of starts for each month",
  machines: "Own machines above the plan",
} as const;
