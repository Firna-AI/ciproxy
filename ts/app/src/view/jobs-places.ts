// Where the waiting jobs of each label can start now, from the facts that the
// API gives and the time of the read. Own machines run a label when one of
// them fits it. Boat starts a job of a label when the label allows Boat, a
// Boat size fits it, and Boat takes new jobs now. Boat starts the job at once
// when no own machine runs its label, and otherwise after the overflow wait.
// Each notice of the live queue reads these places, so no notice promises a
// start that the facts do not confirm. A fact that the API leaves out
// promises nothing.
//
// The fit of each place comes from the label and its resources only, never
// from a hold of Boat. A hold of Boat comes from the overflow facts at the
// time of the read: a saved retry time holds new starts only while it is
// after the read, as the overflow service waits only until that time.

import type {
  JobVmSize,
  JobsLivePage,
  WaitingLabelFit,
} from "../api/jobs-types.ts";

/** Whether an own machine runs the label now, or why none does. */
export type OwnFit =
  /** An allowed own machine fits the label. */
  | "fits"
  /** The label does not run on own machines. */
  | "forbidden"
  /** No own machine takes jobs now. */
  | "no-machine"
  /** Own machines take jobs, and none fits the request of the label. */
  | "no-fit"
  /** The API leaves out the facts of the own machines. */
  | "unknown";

/** Whether a Boat size fits the label, from its permission and the catalog. */
export type BoatFit =
  /** The label does not run on Boat. */
  | { kind: "forbidden" }
  /** No Boat size fits the label. */
  | { kind: "no-fit" }
  /** The API leaves out the Boat facts. */
  | { kind: "unknown" }
  /** A size fits, or the largest size when the label asks for it. */
  | {
      kind: "fits";
      sizeName: string;
      size: JobVmSize;
      largest: boolean;
      /** Boat refused this size, so it starts no job on it now. */
      refused: boolean;
    };

/** Whether Boat takes new jobs now, for every label, at the read time. */
export type BoatHold = {
  /** Boat starts new jobs, a hold stops them, or a fact lacks. */
  state: "takes" | "held" | "uncounted" | "unknown";
  /** The saved retry time, only while it is after the read. */
  retryUntil: string | null;
};

/** Whether Boat starts the jobs of the label now, and on which size. */
export type BoatStart =
  /** The label does not run on Boat. */
  | { kind: "not-allowed" }
  /** No Boat size fits the label. */
  | { kind: "no-size" }
  /** The API leaves out the Boat facts. */
  | { kind: "unknown" }
  /** The Boat costs have no safe count, so Boat starts no new sandbox. */
  | { kind: "uncounted" }
  /** A hold of Boat stops every new start now. */
  | { kind: "held" }
  /** Boat refused the size of this label only. */
  | { kind: "size-refused" }
  /** Boat starts the jobs, on the largest size when nothing else fits. */
  | { kind: "starts"; sizeName: string; size: JobVmSize; largest: boolean };

export type LabelPlace = {
  fit: WaitingLabelFit;
  jobs: number;
  own: OwnFit;
  boatFit: BoatFit;
  boat: BoatStart;
};

/** The hold of Boat at `readAt`, the browser time when the read arrived. */
export function boatHold(page: JobsLivePage, readAt: number): BoatHold {
  const overflow = page.overflow ?? null;
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
  if (!current.label.allow_machines) return "forbidden";
  if (current.fit.fitting_machines > 0) return "fits";
  if (!current.fit.no_machine_fits) return "unknown";
  return current.fit.eligible_machines === 0 ? "no-machine" : "no-fit";
}

function boatFit(fit: WaitingLabelFit): BoatFit {
  const boat = fit.boat;
  switch (boat.state) {
    case "disabled":
      return { kind: "forbidden" };
    case "no_size_fits":
      return { kind: "no-fit" };
    case "unavailable":
      return { kind: "unknown" };
    case "fitting":
    case "largest_fallback":
      return {
        kind: "fits",
        sizeName: boat.size_name,
        size: boat.size,
        largest: boat.state === "largest_fallback",
        refused: boat.refused,
      };
  }
}

function boatStart(fit: BoatFit, hold: BoatHold): BoatStart {
  switch (fit.kind) {
    case "forbidden":
      return { kind: "not-allowed" };
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

/** The place of each waiting label, in the order of the API. */
export function labelPlaces(page: JobsLivePage, hold: BoatHold): LabelPlace[] {
  return page.waiting_labels.map((fit) => {
    const physical = boatFit(fit);
    return {
      fit,
      jobs: Number(fit.waiting_jobs),
      own: ownFit(fit),
      boatFit: physical,
      boat: boatStart(physical, hold),
    };
  });
}

/** No own machine runs the label: it forbids them, or none takes or fits it. */
export function noOwnMachine(own: OwnFit): boolean {
  return own === "forbidden" || own === "no-machine" || own === "no-fit";
}

/**
 * When Boat starts the jobs of a label: at once when no own machine runs the
 * label, after the overflow wait when one does, and never a promise when the
 * own fit is unknown.
 */
export function boatTiming(place: LabelPlace): "at-once" | "after-wait" | null {
  if (place.boat.kind !== "starts") return null;
  if (noOwnMachine(place.own)) return "at-once";
  if (place.own === "fits") return "after-wait";
  return null;
}
