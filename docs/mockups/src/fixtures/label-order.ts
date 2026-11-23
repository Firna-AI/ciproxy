// The order of a label: the places where its jobs run, in order. A place is
// the own machines or one provider. A job goes to the next place of its order
// when the current place cannot take it now, with no extra wait between
// providers, as the answer to item 4 of the provider plan says. The overflow
// wait applies only when a job leaves the own machines. A label can put a
// provider before the own machines, and it can leave them out. A macOS label
// holds the own machines only.

import type { Platform } from "./platform.ts";
import { providerName, type ProviderKey } from "./providers.ts";

/** One place of a label order: the own machines, or one provider. */
export type Place = "machines" | ProviderKey;

/**
 * The order of each Linux label in the main story: the own machines, then
 * Boat, the order that a label had with both of its old switches on.
 */
export const machinesThenBoat: readonly Place[] = ["machines", "boat"];

/** The order of a macOS label, and of a label that holds no provider. */
export const machinesOnly: readonly Place[] = ["machines"];

/** "Your machines", "Your Macs", or the brand of a provider. */
export function placeName(place: Place, platform: Platform = "linux"): string {
  if (place !== "machines") return providerName(place);
  return platform === "macos" ? "Your Macs" : "Your machines";
}

/** "Your machines, then Boat", or "Boat only" for a single place. */
export function orderText(
  places: readonly Place[],
  platform: Platform = "linux",
): string {
  const names = places.map((place) => placeName(place, platform));
  if (names.length === 1) return `${names[0] ?? ""} only`;
  return names.join(", then ");
}

/** The providers of an order, in order. */
export function providersOf(order: readonly Place[]): ProviderKey[] {
  return order.flatMap((place) => (place === "machines" ? [] : [place]));
}
