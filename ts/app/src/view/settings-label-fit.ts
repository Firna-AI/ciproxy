// The fit line of the label dialog, which says, before the save, where jobs
// of the size run now. The placement, the job counts, the offer of each
// machine, the Boat sizes, and whether the workspace has a Boat account come
// from the API fit; this module only puts them into words. Without a Boat
// account, the line keeps the fit of the machines and says that Boat is not
// connected; it never says that a Boat size does not fit unless the Boat
// catalog says so. The sizes in words are in settings-label-sizes.ts.

import type { LabelFit, LabelInput } from "../api/schema.ts";
import {
  labelSizePhrase,
  largestBoat,
  largestBoatBody,
  machineOffer,
} from "./settings-label-sizes.ts";

export type FitLine =
  | { kind: "caption"; text: string }
  | { kind: "warning"; title: string; body: string };

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
  const unreported =
    unknown > 0
      ? ` ${plural(unknown, "machine has", "machines have")} not reported ${unknown === 1 ? "its" : "their"} capacity yet.`
      : "";
  return `${jobs} of this size at a time on ${fitting} of your ${eligible} machines.${unreported}`;
}

const onlyBoat = "Only Boat may run jobs with this label.";
const noBoatFit = "Its Boat fit is not known yet.";
const notConnected = "Boat is not connected.";
const noMachine = "No connected machine can run jobs now.";

/** The label allows Boat, and the API saw that the workspace has no account. */
function boatNotConnected(label: LabelInput, fit: LabelFit): boolean {
  return label.allow_boat && fit.boat_connected === false;
}

/** The machines and the largest Boat size, for a size that nothing fits. */
function offers(fit: LabelFit, machines: boolean, boat: boolean): string {
  const offer = machines ? machineOffer(fit) : null;
  const largest = boat ? largestBoat(fit) : null;
  if (offer !== null && largest !== null) {
    return `${offer}, and the largest Boat size offers ${largest}.`;
  }
  if (offer !== null) return `${offer}.`;
  if (largest !== null) return `The largest Boat size offers ${largest}.`;
  return machines ? noMachine : "Jobs with this label wait until a size fits.";
}

function boatOnly(label: LabelInput, fit: LabelFit): FitLine {
  const fallback = fit.boat.state === "largest_fallback";
  const largest = fallback ? largestBoatBody(fit) : null;
  if (!label.allow_machines) {
    return {
      kind: "caption",
      text: largest === null ? onlyBoat : `${onlyBoat} ${largest}`,
    };
  }
  const size = labelSizePhrase(label);
  if (largest !== null) {
    return {
      kind: "warning",
      title: `No machine and no Boat size fits ${size}`,
      body: largest,
    };
  }
  const offer = machineOffer(fit);
  return {
    kind: "warning",
    title: `No machine fits ${size}`,
    body: `Jobs with this label run on Boat.${offer === null ? "" : ` ${offer}.`}`,
  };
}

const waitForMachine =
  "Jobs with this label wait until a machine that fits connects.";

function waits(label: LabelInput, fit: LabelFit): FitLine {
  const size = labelSizePhrase(label);
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
          title: `No machine fits ${size}`,
          body: `${offers(fit, true, false)} Boat is not connected, so jobs with this label wait until a machine that fits connects.`,
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
      title: `No machine fits ${size}`,
      body: `${offers(fit, true, false)} ${waitForMachine}`,
    };
  }
  // A wait whose Boat fit the catalog does not give names only what is known.
  if (fit.boat.state !== "no_size_fits") {
    return machines
      ? {
          kind: "warning",
          title: `No machine fits ${size}`,
          body: `${offers(fit, true, false)} ${noBoatFit}`,
        }
      : { kind: "caption", text: `${onlyBoat} ${noBoatFit}` };
  }
  return {
    kind: "warning",
    title: machines
      ? `No machine and no Boat size fits ${size}`
      : `No Boat size fits ${size}`,
    body: offers(fit, machines, true),
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
      text: "Your machines have not reported their capacity yet.",
    };
  }
  if (fit.eligible_machines === 0) {
    return { kind: "caption", text: `${noMachine} ${noBoatFit}` };
  }
  return {
    kind: "warning",
    title: `No machine fits ${labelSizePhrase(label)}`,
    body: noBoatFit,
  };
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
