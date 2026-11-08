// The Labels tab: each label with its size, where its jobs run now, and the
// jobs that it ran in the last 30 days. The placement, the count, and whether
// the workspace has a Boat account come from the API; the switches of a label
// only choose the words of a wait. When the API knows the fit of the machines
// but not the Boat fit, the row names both. The fit of the label dialog is in
// settings-label-fit.ts.

import type { LabelFit, LabelInput, LabelView } from "../api/schema.ts";

/**
 * The one label that cannot be deleted, because the warm runners serve it,
 * as "Custom labels" in docs/protocol/labels.md says.
 */
export const protectedLabelName = "ciproxy";

const nbsp = "\u00a0";
const grouped = new Intl.NumberFormat("en-US", { maximumFractionDigits: 0 });

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

export type Placement = {
  /** Where the jobs of the label run now, or why they wait. */
  text: string;
  warn: boolean;
  /** A fact under the placement that the API does not know yet. */
  note: string | null;
};

/** The reason of a wait, by the switches of the label and the account. */
function waitReason(label: LabelInput, fit: LabelFit): string {
  if (!label.allow_machines && !label.allow_boat) {
    return "Your machines and Boat are off";
  }
  if (!label.allow_boat) return "no machine fits";
  if (fit.boat_connected === false) {
    return label.allow_machines ? "no machine fits" : "Boat is not connected";
  }
  if (!label.allow_machines) return "no Boat size fits";
  return "nothing fits";
}

/** Where the jobs of a label run now, as the API places them. */
export function placement(label: LabelInput, fit: LabelFit): Placement {
  switch (fit.placement) {
    case "machines_then_boat":
      return { text: "Your machines, then Boat", warn: false, note: null };
    case "machines_only":
      return { text: "Your machines only", warn: false, note: null };
    case "boat_only":
      return { text: "Boat only", warn: false, note: null };
    case "waits":
      return {
        text: `Waits: ${waitReason(label, fit)}`,
        warn: true,
        note: null,
      };
    case "unavailable":
      return label.allow_machines && fit.fitting_machines > 0
        ? { text: "Your machines", warn: false, note: "Boat fit not known yet" }
        : { text: "Not known yet", warn: false, note: null };
  }
}

export type LabelRow = {
  id: string;
  name: string;
  isDefault: boolean;
  size: string;
  where: Placement;
  /** The jobs of the last 30 days, "14,371", or null when not known. */
  jobs: string | null;
};

export function labelRows(labels: readonly LabelView[]): LabelRow[] {
  return labels.map((view) => {
    const jobs = view.jobs_last_30_days ?? null;
    return {
      id: view.id,
      name: view.label.name,
      isDefault: view.label.name === protectedLabelName,
      size: sizeText(view.label),
      where: placement(view.label, view.fit),
      jobs: jobs === null ? null : grouped.format(BigInt(jobs)),
    };
  });
}

/** "14,371 jobs in 30 days", the count of a label on a phone. */
export function jobsCaption(jobs: string | null): string {
  if (jobs === null) return "Job count not known yet";
  return `${jobs} ${jobs === "1" ? "job" : "jobs"} in 30${nbsp}days`;
}

/** Whether the dialog offers Delete label. */
export function canDelete(label: LabelInput): boolean {
  return label.name !== protectedLabelName;
}
