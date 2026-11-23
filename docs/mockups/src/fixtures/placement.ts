// Where the jobs of a label run at a moment: the places of its order that can
// run its size now, in order. The own machines run a label when one of them
// fits it. A provider runs a label when it can take a job now and one of its
// sizes fits. A provider cannot take a job while the workspace switch is off,
// while it is not connected or not available in the deployment, or while it
// is at its limit, rejected the key, refuses new sandboxes, or waits for
// billing. A place that cannot run the label is skipped.
//
// When no place of the order fits, "When nothing fits" decides: the jobs
// wait, or they run on the largest size of the first provider of the order
// that can take a job and whose largest size holds the image. Only a
// provider that is connected and available has known sizes, so only such a
// provider can be a place that fits for this rule. While the provider that
// would run such a label cannot take a job now, the label waits for its
// reason, as "A size that nothing can run" of labels.md says. That provider
// is the first connected and available provider of the order whose largest
// size holds the image.
//
// Otherwise the jobs wait for the reason of the first place of the order
// that fits the size. For the reason only, a provider that is not connected
// or not available counts as one that fits, because ciproxy reads the sizes
// of a provider from its account in the deployment.

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

/** The same moment while Boat is at its spend limit. */
export const boatAtLimit: PlaceContext = {
  ...boatReady,
  providers: { boat: "limit" },
};

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

/** ciproxy knows the sizes of a provider that is connected and available. */
export function sizesKnown(
  provider: ProviderKey,
  context: PlaceContext,
): boolean {
  const hold = holdOf(provider, context);
  return hold !== "not-connected" && hold !== "unavailable";
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
  const somePlaceFits = label.order.some(
    (place) =>
      place !== "machines" &&
      sizesKnown(place, context) &&
      fitsProvider(place, label),
  );
  if (!somePlaceFits && label.whenNothingFits === "largest-size") {
    const largest = largestProviderOf(label, context);
    if (largest) return { kind: "largest", provider: largest };
    const held = heldLargestOf(label, context);
    if (held) return waits(held);
  }
  return waits(waitReasonOf(label, context));
}

/**
 * The reason of the provider that would run a label on its largest size,
 * while it cannot take a job now: the first connected and available
 * provider of the order whose largest size holds the image.
 */
function heldLargestOf(
  label: Label,
  context: PlaceContext,
): WaitReason | undefined {
  const provider = label.order.find(
    (place): place is ProviderKey =>
      place !== "machines" &&
      sizesKnown(place, context) &&
      largestHoldsImage(place),
  );
  const hold = provider ? holdOf(provider, context) : undefined;
  return provider && hold ? { kind: "held", provider, hold } : undefined;
}

/**
 * The reason of the first place of the order that fits the size. A provider
 * with unknown sizes counts as one that fits, for the reason only.
 */
function waitReasonOf(label: Label, context: PlaceContext): WaitReason {
  for (const place of label.order) {
    if (place === "machines") continue;
    if (sizesKnown(place, context) && !fitsProvider(place, label)) continue;
    const hold = holdOf(place, context);
    if (hold) return { kind: "held", provider: place, hold };
  }
  return { kind: "nothing-fits" };
}
