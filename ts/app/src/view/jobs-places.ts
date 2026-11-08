// Where the waiting jobs of each label can start now, from the facts that the
// API gives. Own machines run a label when one of them fits it. Boat starts a
// job of a label when the label allows Boat, a Boat size fits it, and Boat
// takes new jobs now. Boat starts the job at once when no own machine runs
// its label, and otherwise after the overflow wait. Each notice of the live
// queue reads these places, so no notice promises a start that the facts do
// not confirm. A fact that the API leaves out promises nothing.

import type {
  JobVmSize,
  JobsLivePage,
  WaitingLabelFit,
} from "../api/jobs-types.ts";

/** Whether an own machine runs the label now. */
export type OwnFit = "fits" | "none" | "unknown";

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
  /** Boat takes no new job now, or refuses the size of the label. */
  | { kind: "held" }
  /** Boat starts the jobs, on the largest size when nothing else fits. */
  | { kind: "starts"; sizeName: string; size: JobVmSize; largest: boolean };

export type LabelPlace = {
  fit: WaitingLabelFit;
  jobs: number;
  own: OwnFit;
  boat: BoatStart;
};

/** Whether Boat takes new jobs now, from the overflow facts of the page. */
type BoatNow = "takes" | "held" | "uncounted" | "unknown";

function boatNow(page: JobsLivePage): BoatNow {
  const overflow = page.overflow ?? null;
  if (!page.overflow_enabled) return "held";
  if (overflow === null) return "unknown";
  if (
    !overflow.connected ||
    overflow.key_rejected ||
    overflow.billing_required ||
    overflow.limit_reached === true ||
    (overflow.retry_at ?? null) !== null
  ) {
    return "held";
  }
  if ((overflow.limit_reached ?? null) === null) return "uncounted";
  return "takes";
}

function ownFit(fit: WaitingLabelFit): OwnFit {
  const current = fit.current ?? null;
  if (current === null) return "unknown";
  if (!current.label.allow_machines) return "none";
  if (current.fit.fitting_machines > 0) return "fits";
  return current.fit.no_machine_fits ? "none" : "unknown";
}

function boatStart(fit: WaitingLabelFit, now: BoatNow): BoatStart {
  const boat = fit.boat;
  switch (boat.state) {
    case "disabled":
      return { kind: "not-allowed" };
    case "no_size_fits":
      return { kind: "no-size" };
    case "unavailable":
      return { kind: "unknown" };
    case "fitting":
    case "largest_fallback":
      if (now !== "takes") return { kind: now };
      if (boat.refused) return { kind: "held" };
      return {
        kind: "starts",
        sizeName: boat.size_name,
        size: boat.size,
        largest: boat.state === "largest_fallback",
      };
  }
}

/** The place of each waiting label, in the order of the API. */
export function labelPlaces(page: JobsLivePage): LabelPlace[] {
  const now = boatNow(page);
  return page.waiting_labels.map((fit) => ({
    fit,
    jobs: Number(fit.waiting_jobs),
    own: ownFit(fit),
    boat: boatStart(fit, now),
  }));
}

/**
 * When Boat starts the jobs of a label: at once when no own machine runs the
 * label, after the overflow wait when one does, and never a promise when the
 * own fit is unknown.
 */
export function boatTiming(place: LabelPlace): "at-once" | "after-wait" | null {
  if (place.boat.kind !== "starts") return null;
  if (place.own === "none") return "at-once";
  if (place.own === "fits") return "after-wait";
  return null;
}
