// The places of a label order: own machines, or one provider, in the order
// in which the label tries them. An order holds each place once. Add a place
// offers own machines and each offered provider, which the workspace
// connected once, that the order does not hold. A new Linux label starts
// with own machines, then each offered provider. A macOS label holds own
// machines only.

import type {
  LabelPlace,
  OverflowProvider,
  Platform,
  ProviderSettings,
} from "../api/schema.ts";
import { providerName } from "./providers.ts";

export const machinePlace: LabelPlace = { kind: "machine" };

export function providerPlace(provider: OverflowProvider): LabelPlace {
  return { kind: "overflow", provider };
}

/** "machine" or the provider: one key for each place of an order. */
export function placeKey(place: LabelPlace): string {
  return place.kind === "machine" ? "machine" : place.provider;
}

export function samePlace(a: LabelPlace, b: LabelPlace): boolean {
  return placeKey(a) === placeKey(b);
}

export function hasMachines(places: readonly LabelPlace[]): boolean {
  return places.some((place) => place.kind === "machine");
}

/** The providers of an order, in order. */
export function orderProviders(
  places: readonly LabelPlace[],
): OverflowProvider[] {
  return places.flatMap((place) =>
    place.kind === "overflow" ? [place.provider] : [],
  );
}

/** "Your machines", "Your Macs", or the brand of a provider. */
export function placeName(place: LabelPlace, platform: Platform): string {
  if (place.kind === "overflow") return providerName(place.provider);
  return platform === "macos_arm64" ? "Your Macs" : "Your machines";
}

/** "Your machines, then Boat", or "Boat only" for a single place. */
export function orderText(
  places: readonly LabelPlace[],
  platform: Platform,
): string {
  const names = places.map((place) => placeName(place, platform));
  if (names.length === 1) return `${names[0] ?? ""} only`;
  return names.join(", then ");
}

/** The providers that the workspace offered, in the order of the API. */
export function offeredProviders(
  providers: readonly ProviderSettings[],
): OverflowProvider[] {
  return providers
    .filter((provider) => provider.offered)
    .map((provider) => provider.provider);
}

/** The order of a new Linux label: own machines, then each offered provider. */
export function defaultOrder(
  offered: readonly OverflowProvider[],
): LabelPlace[] {
  return [machinePlace, ...offered.map(providerPlace)];
}

/** The places that Add a place offers: those that the order does not hold. */
export function addablePlaces(
  order: readonly LabelPlace[],
  offered: readonly OverflowProvider[],
): LabelPlace[] {
  return defaultOrder(offered).filter(
    (place) => !order.some((held) => samePlace(held, place)),
  );
}
