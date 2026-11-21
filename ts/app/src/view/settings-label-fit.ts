// The fit line of the label dialog, which says, before the save, where jobs
// of the size run now. The placement, the job counts, the Boat fit, and
// whether the workspace has a Boat account come from the API fit; this module
// only puts them into words. A size that no machine fits is named by its full
// request, never by what a machine or a Boat size offers. Without a Boat
// account, the line keeps the fit of the machines and says that Boat is not
// connected; it never says that a Boat size does not fit unless the Boat
// catalog says so. The sizes in words are in settings-label-sizes.ts.

import type { LabelFit, LabelInput } from "../api/schema.ts";
import { largestBoatText, requestPhrase } from "./settings-label-sizes.ts";

/** A caption, or a warning whose title can stand alone. */
export type FitLine =
  | { kind: "caption"; text: string }
  | { kind: "warning"; title: string; body: string | null };

/** "1 job", "3 jobs", or "2 to 3 jobs". */
export function jobsText(minimum: string, maximum: string): string {
  const unit = Number(maximum) === 1 ? "job" : "jobs";
  return minimum === maximum
    ? `${maximum} ${unit}`
    : `${minimum} to ${maximum} ${unit}`;
}

function plural(count: number, one: string, many: string): string {
  return `${count} ${count === 1 ? one : many}`;
}

/** The jobs of the size that each fitting machine runs at a time. */
function machineFit(fit: LabelFit): string {
  const { eligible_machines: eligible, fitting_machines: fitting } = fit;
  const unknown = fit.unavailable_machines;
  const minimum = fit.minimum_jobs ?? null;
  const maximum = fit.maximum_jobs ?? null;
  if (minimum === null || maximum === null) {
    return "Your machines run jobs of this size.";
  }
  const jobs = jobsText(minimum, maximum);
  if (fitting === eligible) {
    return `Each of your machines runs ${jobs} of this size at a time.`;
  }
  const unknownCapacity =
    unknown > 0
      ? ` The capacity of ${plural(unknown, "machine", "machines")} is not known yet.`
      : "";
  return `${jobs} of this size at a time on ${fitting} of your ${eligible} machines.${unknownCapacity}`;
}

const onlyBoat = "Only Boat may run jobs with this label.";
const noBoatFit = "Its Boat fit is not known yet.";
const notConnected = "Boat is not connected.";
const noMachine = "No connected machine can run jobs now.";
const waitForMachine =
  "Jobs with this label wait until a machine that fits connects.";

/** The label allows Boat, and the API saw that the workspace has no account. */
function boatNotConnected(label: LabelInput, fit: LabelFit): boolean {
  return label.allow_boat && fit.boat_connected === false;
}

/** The title of a size that no machine fits, with its full request. */
function noMachineFits(label: LabelInput): string {
  return `No machine of yours fits a job of ${requestPhrase(label)}`;
}

/** The title of a size that no machine and no Boat size fits. */
function nothingFits(label: LabelInput): string {
  return `No machine of yours and no Boat size fits a job of ${requestPhrase(label)}`;
}

/** The sentence, after the fact that no machine is connected when none is. */
function afterMachines(fit: LabelFit, sentence: string): string {
  return fit.eligible_machines === 0 ? `${noMachine} ${sentence}` : sentence;
}

function boatOnly(label: LabelInput, fit: LabelFit): FitLine {
  const fallback = fit.boat.state === "largest_fallback";
  if (!label.allow_machines) {
    return {
      kind: "caption",
      text: fallback ? `${onlyBoat} ${largestBoatText}` : onlyBoat,
    };
  }
  return fallback
    ? { kind: "warning", title: nothingFits(label), body: largestBoatText }
    : {
        kind: "warning",
        title: noMachineFits(label),
        body: "Jobs with this label run on Boat.",
      };
}

function waits(label: LabelInput, fit: LabelFit): FitLine {
  const { allow_machines: machines, allow_boat: boat } = label;
  if (!machines && !boat) {
    return {
      kind: "warning",
      title: "Turn on Your machines or Boat",
      body: "Jobs with this label wait until one of them is on.",
    };
  }
  if (boatNotConnected(label, fit)) {
    return machines
      ? {
          kind: "warning",
          title: noMachineFits(label),
          body: afterMachines(
            fit,
            "Boat is not connected, so jobs with this label wait until a machine that fits connects.",
          ),
        }
      : {
          kind: "warning",
          title: "Boat is not connected",
          body: "Only Boat may run jobs with this label, so they wait until you connect Boat.",
        };
  }
  if (!boat) {
    return {
      kind: "warning",
      title: noMachineFits(label),
      body: afterMachines(fit, waitForMachine),
    };
  }
  // A wait whose Boat fit the catalog does not give names only what is known.
  if (fit.boat.state !== "no_size_fits") {
    return machines
      ? {
          kind: "warning",
          title: noMachineFits(label),
          body: afterMachines(fit, noBoatFit),
        }
      : { kind: "caption", text: `${onlyBoat} ${noBoatFit}` };
  }
  if (!machines) {
    return {
      kind: "warning",
      title: `No Boat size fits a job of ${requestPhrase(label)}`,
      body: null,
    };
  }
  return {
    kind: "warning",
    title: nothingFits(label),
    body: fit.eligible_machines === 0 ? noMachine : null,
  };
}

function unavailable(label: LabelInput, fit: LabelFit): FitLine {
  if (!label.allow_machines) {
    return { kind: "caption", text: `${onlyBoat} ${noBoatFit}` };
  }
  if (fit.fitting_machines > 0) {
    return { kind: "caption", text: `${machineFit(fit)} ${noBoatFit}` };
  }
  if (fit.unavailable_machines > 0) {
    return {
      kind: "caption",
      text: "The capacity of your machines is not known yet.",
    };
  }
  if (fit.eligible_machines === 0) {
    return { kind: "caption", text: `${noMachine} ${noBoatFit}` };
  }
  return { kind: "warning", title: noMachineFits(label), body: noBoatFit };
}

/** The line of the dialog that says, before the save, where jobs run. */
export function fitLine(label: LabelInput, fit: LabelFit): FitLine {
  switch (fit.placement) {
    case "machines_then_boat":
      return { kind: "caption", text: machineFit(fit) };
    case "machines_only":
      return {
        kind: "caption",
        text: boatNotConnected(label, fit)
          ? `${machineFit(fit)} ${notConnected}`
          : machineFit(fit),
      };
    case "boat_only":
      return boatOnly(label, fit);
    case "waits":
      return waits(label, fit);
    case "unavailable":
      return unavailable(label, fit);
  }
}
