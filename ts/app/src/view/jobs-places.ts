// Where the waiting jobs of each label can start now, from the facts that the
// API gives and the time of the read. Own machines run a label when the
// order holds them and one of them fits it. A provider starts a job of a
// label when the order holds it, one of its sizes fits, and it takes new
// jobs now. A job goes to the first place of its order that can take it:
// a provider before own machines starts it at once, and a provider after own
// machines that fit starts it after the overflow wait. Each notice of the
// live queue reads these places, so no notice promises a start that the
// facts do not confirm. A fact that the API leaves out promises nothing.
//
// The fit of each place comes from the label and its resources only, never
// from a hold of a provider. A hold comes from the overflow facts of its
// provider at the time of the read: a saved retry time holds new starts only
// while it is after the read, as the overflow service waits only until that
// time.

import type {
  JobOverflow,
  JobVmSize,
  JobsLivePage,
  WaitingLabelFit,
} from "../api/jobs-types.ts";
import type {
  LabelRouting,
  OverflowProvider,
  ProviderLabelFit,
} from "../api/schema.ts";
import { hasMachines } from "./settings-label-places.ts";

/** Whether an own machine runs the label now, or why none does. */
export type OwnFit =
  /** The order holds own machines, and one of them fits the label. */
  | "fits"
  /** The order of the label does not hold own machines. */
  | "forbidden"
  /** No own machine takes jobs now. */
  | "no-machine"
  /** Own machines take jobs, and none fits the request of the label. */
  | "no-fit"
  /** The API leaves out the facts of the own machines. */
  | "unknown";

/** Whether a size of a provider of the order fits the label. */
export type ProviderFit =
  /** No size of the provider fits the label. */
  | { kind: "no-fit" }
  /** The API leaves out the facts of the provider. */
  | { kind: "unknown" }
  /** A size fits, or the largest size when the label asks for it. */
  | {
      kind: "fits";
      sizeName: string;
      size: JobVmSize;
      largest: boolean;
      /** The provider refused this size, so it starts no job on it now. */
      refused: boolean;
    };

/** Whether a provider takes new jobs now, for every label, at the read time. */
export type ProviderHoldState = {
  /** It starts new jobs, a hold stops them, or a fact lacks. */
  state: "takes" | "held" | "uncounted" | "unknown";
  /** The saved retry time, only while it is after the read. */
  retryUntil: string | null;
};

/** Whether a provider starts the jobs of the label now, and on which size. */
export type ProviderStart =
  /** No size of the provider fits the label. */
  | { kind: "no-size" }
  /** The API leaves out the facts of the provider. */
  | { kind: "unknown" }
  /** The costs have no safe count, so the provider starts no new sandbox. */
  | { kind: "uncounted" }
  /** A hold of the provider stops every new start now. */
  | { kind: "held" }
  /** The provider refused the size of this label only. */
  | { kind: "size-refused" }
  /** It starts the jobs, on its largest size when nothing else fits. */
  | { kind: "starts"; sizeName: string; size: JobVmSize; largest: boolean };

/** One provider of the order of a label, with its fit and its start. */
export type ProviderPlace = {
  provider: OverflowProvider;
  fit: ProviderFit;
  start: ProviderStart;
  /** The provider comes before own machines in the order. */
  beforeOwn: boolean;
};

export type LabelPlace = {
  fit: WaitingLabelFit;
  jobs: number;
  own: OwnFit;
  /** The providers of the order, in order. */
  providers: readonly ProviderPlace[];
  /** Where the API routes the label now, when the label still exists. */
  routing: LabelRouting | null;
};

/** The observation of one provider in the live queue, if the API has one. */
export function observation(
  page: JobsLivePage,
  provider: OverflowProvider,
): JobOverflow | null {
  return page.overflow.find((item) => item.provider === provider) ?? null;
}

/** The hold of a provider at `readAt`, the browser time of the read. */
export function providerHold(
  page: JobsLivePage,
  provider: OverflowProvider,
  readAt: number,
): ProviderHoldState {
  const overflow = observation(page, provider);
  const retry = overflow?.retry_at ?? null;
  const retryUntil =
    retry !== null && Date.parse(retry) > readAt ? retry : null;
  if (!page.overflow_enabled) return { state: "held", retryUntil };
  if (overflow === null) return { state: "unknown", retryUntil: null };
  if (
    !overflow.connected ||
    overflow.key_rejected ||
    overflow.billing_required ||
    overflow.limit_reached === true ||
    retryUntil !== null
  ) {
    return { state: "held", retryUntil };
  }
  if ((overflow.limit_reached ?? null) === null) {
    return { state: "uncounted", retryUntil: null };
  }
  return { state: "takes", retryUntil: null };
}

function ownFit(fit: WaitingLabelFit): OwnFit {
  const current = fit.current ?? null;
  if (current === null) return "unknown";
  if (!hasMachines(current.label.order)) return "forbidden";
  if (current.fit.fitting_machines > 0) return "fits";
  if (!current.fit.no_machine_fits) return "unknown";
  return current.fit.eligible_machines === 0 ? "no-machine" : "no-fit";
}

function providerFit(fit: ProviderLabelFit["fit"]): ProviderFit {
  switch (fit.state) {
    case "disabled":
    case "unavailable":
      return { kind: "unknown" };
    case "no_size_fits":
      return { kind: "no-fit" };
    case "fitting":
    case "largest_fallback":
      return {
        kind: "fits",
        sizeName: fit.size_name,
        size: fit.size,
        largest: fit.state === "largest_fallback",
        refused: fit.refused,
      };
  }
}

function providerStart(
  fit: ProviderFit,
  hold: ProviderHoldState,
): ProviderStart {
  switch (fit.kind) {
    case "no-fit":
      return { kind: "no-size" };
    case "unknown":
      return { kind: "unknown" };
    case "fits":
      if (hold.state !== "takes") return { kind: hold.state };
      if (fit.refused) return { kind: "size-refused" };
      return {
        kind: "starts",
        sizeName: fit.sizeName,
        size: fit.size,
        largest: fit.largest,
      };
  }
}

/**
 * The providers of the order of a label, in order. The API lists the
 * providers of the order first and marks every other provider as disabled.
 */
function orderedProviders(
  fit: WaitingLabelFit,
  holds: (provider: OverflowProvider) => ProviderHoldState,
): ProviderPlace[] {
  const order = fit.current?.label.order ?? null;
  const machineIndex =
    order?.findIndex((place) => place.kind === "machine") ?? -1;
  return fit.providers.flatMap((item) => {
    if (item.fit.state === "disabled") return [];
    const index =
      order?.findIndex(
        (place) =>
          place.kind === "overflow" && place.provider === item.provider,
      ) ?? -1;
    if (order !== null && index < 0) return [];
    const physical = providerFit(item.fit);
    return [
      {
        provider: item.provider,
        fit: physical,
        start: providerStart(physical, holds(item.provider)),
        beforeOwn: machineIndex >= 0 && index >= 0 && index < machineIndex,
      },
    ];
  });
}

/** The place of each waiting label, in the order of the API. */
export function labelPlaces(page: JobsLivePage, readAt: number): LabelPlace[] {
  const holds = (provider: OverflowProvider) =>
    providerHold(page, provider, readAt);
  return page.waiting_labels.map((fit) => ({
    fit,
    jobs: Number(fit.waiting_jobs),
    own: ownFit(fit),
    providers: orderedProviders(fit, holds),
    routing: fit.current?.fit.routing ?? null,
  }));
}
