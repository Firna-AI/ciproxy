// The Labels tab: each label with its size and where its jobs can run, and
// the fit line of the label dialog, which states how many jobs of the size
// each connected machine runs at a time. Every count comes from the API fit.
// The API gives no Boat fit, so the Boat part states only the switch of the
// label, a policy, and says that the Boat fit is not available.

import type { LabelFit, LabelInput, LabelView } from "../api/schema.ts";

/**
 * The one label that cannot be deleted, because the warm runners serve it,
 * as "Custom labels" in docs/protocol/labels.md says.
 */
export const protectedLabelName = "ciproxy";

const nbsp = " ";

/** "8 vCPU · 32 GB · 60 GB free disk", each value whole on one line. */
export function sizeText(label: LabelInput): string {
  return [
    `${label.vcpus} vCPU`,
    `${label.memory_gb} GB`,
    `${label.disk_gb} GB free disk`,
  ]
    .map((part) => part.replaceAll(" ", nbsp))
    .join(`${nbsp}· `);
}

/** The Boat part of a label that allows Boat: a policy, not a fit. */
export const boatAllowed = `Boat allowed${nbsp}· Boat fit not available`;

export type Placement = {
  /** What the fit says about your machines, or that the jobs wait. */
  text: string;
  warn: boolean;
  /** The Boat switch as a policy, when the jobs can still go to Boat. */
  boat: string | null;
};

/** Where the jobs of a label can run, from its switches and its fit. */
export function placement(label: LabelInput, fit: LabelFit): Placement {
  const { allow_machines: machines, allow_boat: allowBoat } = label;
  const boat = allowBoat ? boatAllowed : null;
  if (!machines) {
    return allowBoat
      ? { text: "Your machines off", warn: false, boat }
      : { text: "Waits: Your machines and Boat are off", warn: true, boat };
  }
  if (fit.fitting_machines > 0) {
    return { text: "Your machines", warn: false, boat };
  }
  if (fit.unavailable_machines > 0 && !fit.no_machine_fits) {
    return { text: "Machine capacity not known yet", warn: false, boat };
  }
  return allowBoat
    ? { text: "No machine fits", warn: false, boat }
    : { text: "Waits: no machine fits", warn: true, boat };
}

export type LabelRow = {
  id: string;
  name: string;
  isDefault: boolean;
  size: string;
  where: Placement;
};

export function labelRows(labels: readonly LabelView[]): LabelRow[] {
  return labels.map((view) => ({
    id: view.id,
    name: view.label.name,
    isDefault: view.label.name === protectedLabelName,
    size: sizeText(view.label),
    where: placement(view.label, view.fit),
  }));
}

/** Whether the dialog offers Delete label. */
export function canDelete(label: LabelInput): boolean {
  return label.name !== protectedLabelName;
}

function plural(count: number, one: string, many: string): string {
  return `${count} ${count === 1 ? one : many}`;
}

/** "1 job", "3 jobs", or "2 to 3 jobs". */
export function jobsText(minimum: string, maximum: string): string {
  const most = Number(maximum);
  const unit = most === 1 ? "job" : "jobs";
  return minimum === maximum
    ? `${maximum} ${unit}`
    : `${minimum} to ${maximum} ${unit}`;
}

/** The routing policy of a label that only Boat may run. */
const onlyBoat = "Only Boat may run jobs with this label.";

/** The API gives no Boat fit, so the dialog says that it is not available. */
const noBoatFit = "Its Boat fit is not available.";

export type FitLine =
  | { kind: "caption"; text: string }
  | { kind: "warning"; title: string; body: string };

/** The line of the dialog that says, before the save, where jobs run. */
export function fitLine(label: LabelInput, fit: LabelFit): FitLine {
  const { eligible_machines: eligible, fitting_machines: fitting } = fit;
  const unknown = fit.unavailable_machines;
  if (!label.allow_machines) {
    return label.allow_boat
      ? { kind: "caption", text: `${onlyBoat} ${noBoatFit}` }
      : {
          kind: "warning",
          title: "Turn on Your machines or Boat",
          body: "Jobs with this label wait until one of them is on.",
        };
  }
  const unreported =
    unknown > 0
      ? ` ${plural(unknown, "machine has", "machines have")} not reported ${unknown === 1 ? "its" : "their"} capacity yet.`
      : "";
  const minimum = fit.minimum_jobs ?? null;
  const maximum = fit.maximum_jobs ?? null;
  if (fitting > 0 && minimum !== null && maximum !== null) {
    const jobs = jobsText(minimum, maximum);
    const text =
      fitting === eligible
        ? `Each of your machines runs ${jobs} of this size at a time.`
        : `${jobs} of this size at a time on ${fitting} of your ${eligible} machines.${unreported}`;
    return { kind: "caption", text };
  }
  if (eligible === 0) {
    return { kind: "caption", text: "No connected machine can run jobs now." };
  }
  if (!fit.no_machine_fits) {
    return {
      kind: "caption",
      text: "Your machines have not reported their capacity yet.",
    };
  }
  return {
    kind: "warning",
    title: "No machine fits this size",
    body: label.allow_boat
      ? `${onlyBoat} ${noBoatFit}`
      : "Jobs with this label wait until a machine that fits connects.",
  };
}
