// The words and formats of billing values: plan names, a monthly price, the
// capacity of a plan, its add-ons, and the UTC days of billing times. Every
// number comes from the API; these helpers only format it.

import type {
  PlanAddOns,
  PlanOffer,
  PlanSelection,
  PlanTier,
} from "../api/billing-types.ts";
import {
  formatCents,
  formatCount,
  formatUtcDate,
  unbroken,
  utcDayOf,
} from "./exact-format.ts";

export const tierNames: Readonly<Record<PlanTier, string>> = {
  free: "Free",
  team: "Team",
  business: "Business",
};

/** The words in place of a payment action that the role does not allow. */
export const askPayer = "Ask an Admin or a Billing person";

/** "$49 a month", whole on one line. */
export function monthlyText(cents: string): string {
  return unbroken(`${formatCents(cents)} a month`);
}

/** "1 own machine", "3 own machines". */
export function machinesText(count: string): string {
  return `${formatCount(count)} own ${count === "1" ? "machine" : "machines"}`;
}

/** "25,000 starts a month and 3 own machines": what a plan includes. */
export function capacityText(offer: PlanOffer): string {
  return `${formatCount(offer.included_starts)} starts a month and ${machinesText(offer.included_machines)}`;
}

/** "1 block of starts", "2 blocks of starts". */
export function startBlocksText(blocks: number): string {
  return `${blocks} ${blocks === 1 ? "block" : "blocks"} of starts`;
}

/** "1 extra own machine", "2 extra own machines". */
export function extraMachinesText(machines: number): string {
  return `${machines} extra own ${machines === 1 ? "machine" : "machines"}`;
}

/** A saved add-on quantity: "None", "1 block of starts", "2 extra own machines". */
export function formatAddOn(
  kind: "starts" | "machines",
  count: number,
): string {
  if (count === 0) return "None";
  return kind === "starts" ? startBlocksText(count) : extraMachinesText(count);
}

export function hasAddOns(addOns: PlanAddOns): boolean {
  return addOns.start_blocks > 0 || addOns.machines > 0;
}

/** "Team", or "Team with 1 block of starts and 1 extra own machine". */
export function selectionText(selection: PlanSelection): string {
  const { start_blocks: blocks, machines } = selection.add_ons;
  const parts = [
    ...(blocks > 0 ? [startBlocksText(blocks)] : []),
    ...(machines > 0 ? [extraMachinesText(machines)] : []),
  ];
  const name = tierNames[selection.tier];
  return parts.length === 0 ? name : `${name} with ${parts.join(" and ")}`;
}

/** The UTC day of an API time: "2026-12-04T00:00:00Z" -> "4 December". */
export function dayText(time: string): string {
  return formatUtcDate(utcDayOf(Date.parse(time)));
}

/**
 * The last UTC day before an exclusive end: an end at midnight on 4 December
 * gives "3 December".
 */
export function lastDayText(end: string): string {
  return formatUtcDate(utcDayOf(Date.parse(end) - 1));
}

/** "1st", "2nd", "3rd", "4th", "11th", "22nd": the day of a monthly charge. */
export function ordinalDay(day: number): string {
  const teen = day % 100 >= 11 && day % 100 <= 13;
  const suffix = teen
    ? "th"
    : (({ 1: "st", 2: "nd", 3: "rd" } as const)[day % 10] ?? "th");
  return `${day}${suffix}`;
}
