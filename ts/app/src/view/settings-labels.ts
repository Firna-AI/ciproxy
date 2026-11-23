// The Labels tab: each label with its platform and size, where its jobs run
// now, and the
// jobs that it ran in the last 30 days. A macOS label runs on the Macs only.
// The placement, the count, and whether
// the workspace has a Boat account come from the API; the switches of a label
// only choose the words of a wait. When the API knows the fit of the machines
// but not the Boat fit, the row names both. The fit of the label dialog is in
// settings-label-fit.ts. The list shows the Linux labels first, then the
// macOS labels, each from the smallest size to the largest.

import type {
  LabelFit,
  LabelInput,
  LabelView,
  Platform,
} from "../api/schema.ts";
import { isMacLabel, labelPlatform, platformName } from "./platform.ts";
import { boatLabelFit, hasBoat, hasMachines } from "./settings-label-places.ts";

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
  if (!hasMachines(label.order) && !hasBoat(label.order)) {
    return "Your machines and Boat are off";
  }
  if (!hasBoat(label.order)) return "no machine fits";
  if (boatLabelFit(fit.providers)?.connected === false) {
    return hasMachines(label.order)
      ? "no machine fits"
      : "Boat is not connected";
  }
  if (!hasMachines(label.order)) return "no Boat size fits";
  return "nothing fits";
}

/** Where the jobs of a macOS label run now: on the Macs only, or they wait. */
function macPlacement(fit: LabelFit): Placement {
  if (fit.fitting_places?.length === 0) {
    return { text: "Waits: no Mac fits", warn: true, note: null };
  }
  if (fit.fitting_places == null && fit.fitting_machines === 0) {
    return { text: "Not known yet", warn: false, note: null };
  }
  return { text: "Your Macs only", warn: false, note: null };
}

/** Where the jobs of a label run now, as the API places them. */
export function placement(label: LabelInput, fit: LabelFit): Placement {
  if (isMacLabel(label)) return macPlacement(fit);
  const places = fit.fitting_places ?? null;
  if (places === null) {
    return hasMachines(label.order) && fit.fitting_machines > 0
      ? { text: "Your machines", warn: false, note: "Boat fit not known yet" }
      : { text: "Not known yet", warn: false, note: null };
  }
  if (hasMachines(places)) {
    return {
      text: hasBoat(places) ? "Your machines, then Boat" : "Your machines only",
      warn: false,
      note: null,
    };
  }
  if (hasBoat(places)) return { text: "Boat only", warn: false, note: null };
  return { text: `Waits: ${waitReason(label, fit)}`, warn: true, note: null };
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

/** Linux labels come before macOS labels in the list. */
const platformRank: Readonly<Record<Platform, number>> = {
  linux_x64: 0,
  macos_arm64: 1,
};

/** Two names by character code, so the order is the same in every locale. */
function byCharacterCode(a: string, b: string): number {
  if (a === b) return 0;
  return a < b ? -1 : 1;
}

/**
 * The order of the list: by platform, then vCPU, memory, free disk, and
 * name, as "The label list" in docs/protocol/labels.md says.
 */
export function compareLabels(a: LabelInput, b: LabelInput): number {
  return (
    platformRank[labelPlatform(a)] - platformRank[labelPlatform(b)] ||
    a.vcpus - b.vcpus ||
    a.memory_gb - b.memory_gb ||
    a.disk_gb - b.disk_gb ||
    byCharacterCode(a.name, b.name)
  );
}

/** The rows of the list, in the order of compareLabels. */
export function labelRows(labels: readonly LabelView[]): LabelRow[] {
  const ordered = [...labels].sort((a, b) => compareLabels(a.label, b.label));
  return ordered.map((view) => {
    const jobs = view.jobs_last_30_days ?? null;
    return {
      id: view.id,
      name: view.label.name,
      isDefault: view.label.name === protectedLabelName,
      size: `${platformName[labelPlatform(view.label)]}${nbsp}· ${sizeText(view.label)}`,
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
