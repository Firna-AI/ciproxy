// Where the jobs of a label run at a moment: the places of its order that can
// run its size now, in order. The own machines run a label when one of them
// fits it. A provider runs a label when it can take a job now and one of its
// sizes fits. A provider cannot take a job while the workspace switch is off,
// while it is not connected or not available in the deployment, or while it
// is at its limit, rejected the key, refuses new sandboxes, or waits for
// billing. A place that cannot run the label is skipped. When no place can
// run it, the label waits for the reason of the first place of its order
// that fits its size. ciproxy reads the sizes of a provider from its
// account, so a provider that is not connected counts as one that fits. When no place fits, "When nothing fits" decides: the
// jobs wait, or they run on the largest size of the first provider of the
// order that can take a job and whose largest size holds the image.

import type { Place } from "./label-order.ts";
import { machineScenes, type Machine } from "./machines.ts";
import type { ProviderKey } from "./providers.ts";
import { fitsOwnMachine, type Label } from "./settings.ts";
import { fitsProvider, largestHoldsImage } from "./sizes.ts";

/** The state of a connected provider at a moment. */
export type ProviderState =
  "ready" | "unavailable" | "limit" | "rejected" | "refusing" | "billing";

/** Why a provider cannot take a job now. */
export type ProviderHold =
  Exclude<ProviderState, "ready"> | "off" | "not-connected";

/** What decides where the jobs of a label run at a moment. */
export type PlaceContext = {
  /** The workspace switch "Use overflow providers". */
  overflowOn: boolean;
  /** The state of each connected provider. Others are not connected. */
  providers: Readonly<Partial<Record<ProviderKey, ProviderState>>>;
  /** The own machines of the moment. */
  machines: readonly Machine[];
};

const steady = machineScenes.steady;

/** The busy afternoon of 18 November: Boat takes jobs. */
export const boatReady: PlaceContext = {
  overflowOn: true,
  providers: { boat: "ready" },
  machines: steady,
};

/** The same moment without a connected provider, as after a skipped setup. */
export const noProviderReady: PlaceContext = {
  overflowOn: true,
  providers: {},
  machines: steady,
};

/** The same moment in the what-ifs with two providers. */
export const twoReady: PlaceContext = {
  overflowOn: true,
  providers: { boat: "ready", second: "ready" },
  machines: steady,
};

/** Two providers, while the deployment has no settings for Boat. */
export const boatUnavailable: PlaceContext = {
  ...twoReady,
  providers: { boat: "unavailable", second: "ready" },
};

/** The busy afternoon while "Use overflow providers" is off. */
export const overflowOff: PlaceContext = { ...boatReady, overflowOn: false };

/** Why a provider cannot take a job now, or nothing when it can. */
export function holdOf(
  provider: ProviderKey,
  context: PlaceContext,
): ProviderHold | undefined {
  if (!context.overflowOn) return "off";
  const state = context.providers[provider];
  if (state === undefined) return "not-connected";
  return state === "ready" ? undefined : state;
}

/** Why the jobs of a label wait. */
export type WaitReason =
  | { kind: "empty-order" }
  | { kind: "nothing-fits" }
  | { kind: "held"; provider: ProviderKey; hold: ProviderHold };

/** Where the jobs of a label run now. */
export type Placement =
  | { kind: "places"; places: readonly Place[] }
  | { kind: "largest"; provider: ProviderKey }
  | { kind: "waits"; reason: WaitReason };

/** The place can run a job of the label now. */
export function placeRuns(
  place: Place,
  label: Label,
  context: PlaceContext,
): boolean {
  if (place === "machines") return fitsOwnMachine(label, context.machines);
  return holdOf(place, context) === undefined && fitsProvider(place, label);
}

/** The provider whose largest size runs a label that nothing fits. */
function largestProviderOf(
  label: Label,
  context: PlaceContext,
): ProviderKey | undefined {
  return label.order.find(
    (place): place is ProviderKey =>
      place !== "machines" &&
      holdOf(place, context) === undefined &&
      largestHoldsImage(place),
  );
}

function waits(reason: WaitReason): Placement {
  return { kind: "waits", reason };
}

export function placementOf(label: Label, context: PlaceContext): Placement {
  if (label.order.length === 0) return waits({ kind: "empty-order" });
  const places = label.order.filter((place) =>
    placeRuns(place, label, context),
  );
  if (places.length > 0) return { kind: "places", places };
  for (const place of label.order) {
    if (place === "machines") continue;
    const hold = holdOf(place, context);
    if (hold !== "not-connected" && !fitsProvider(place, label)) continue;
    if (hold) return waits({ kind: "held", provider: place, hold });
  }
  const largest =
    label.whenNothingFits === "largest-size"
      ? largestProviderOf(label, context)
      : undefined;
  return largest
    ? { kind: "largest", provider: largest }
    : waits({ kind: "nothing-fits" });
}
