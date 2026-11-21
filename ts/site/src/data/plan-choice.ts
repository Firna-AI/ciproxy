// A choice of a tier and add-on quantities, as the offer at a soft limit
// picks it: the cheapest tier with add-ons that includes the starts and the
// own machines of a use.

import { formatCount } from "./format.ts";
import { addOnPrices, tierOrder, tiers, type Tier } from "./tiers.ts";

/** A tier with the add-on quantities of a choice, and its monthly price. */
export type PlanChoice = {
  tier: Tier;
  extraStartBlocks: number;
  extraMachines: number;
  price: number;
};

/**
 * The cheapest tier with add-ons that includes these starts and own machines
 * a month. A tier without add-ons qualifies only within its own limits.
 */
export function cheapestPlan(starts: number, machines: number): PlanChoice {
  const choices = tierOrder.flatMap((key): PlanChoice[] => {
    const tier = tiers[key];
    const extraStartBlocks = Math.max(
      0,
      Math.ceil((starts - tier.starts) / addOnPrices.startsBlock),
    );
    const extraMachines = Math.max(0, machines - tier.machines);
    const needsAddOns = extraStartBlocks > 0 || extraMachines > 0;
    if (needsAddOns && !tier.addOns) return [];
    const price =
      tier.price +
      extraStartBlocks * addOnPrices.startsPrice +
      extraMachines * addOnPrices.machinePrice;
    return [{ tier, extraStartBlocks, extraMachines, price }];
  });
  // The largest tier sells add-ons, so every use has at least one choice.
  return choices.reduce((best, choice) =>
    choice.price < best.price ? choice : best,
  );
}

/** "Team", or "Team with 30,000 extra starts" for a choice with add-ons. */
export function planChoiceText(choice: PlanChoice): string {
  const extras = [
    choice.extraStartBlocks > 0
      ? `${formatCount(choice.extraStartBlocks * addOnPrices.startsBlock)} extra starts`
      : "",
    choice.extraMachines > 0
      ? `${choice.extraMachines} extra own ${choice.extraMachines === 1 ? "machine" : "machines"}`
      : "",
  ].filter(Boolean);
  return extras.length > 0
    ? `${choice.tier.name} with ${extras.join(" and ")}`
    : choice.tier.name;
}
