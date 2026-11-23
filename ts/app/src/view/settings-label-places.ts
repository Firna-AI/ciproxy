// Place selection for the current machine and Boat controls.

import type { LabelPlace, ProviderLabelFit } from "../api/schema.ts";

export function hasMachines(places: readonly LabelPlace[]): boolean {
  return places.some((place) => place.kind === "machine");
}

export function hasBoat(places: readonly LabelPlace[]): boolean {
  return places.some(
    (place) => place.kind === "overflow" && place.provider === "boat",
  );
}

export function boatLabelFit(
  providers: readonly ProviderLabelFit[],
): ProviderLabelFit | undefined {
  return providers.find((provider) => provider.provider === "boat");
}
