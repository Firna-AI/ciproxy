// The Labels tab: each label with its platform and size, where its jobs run
// now, and the jobs that it ran in the last 30 days. Where jobs run names the
// places of its order that can run the label now, in order, so a provider
// that cannot take a job or has no size that fits drops out. A label that no
// place can run now waits, with the reason of the first place of its order
// that fits its size. The API decides the places and the reason with the
// routing of the label, which the Jobs notices read too; this module only
// puts them into words. A macOS label runs on the Macs only. The fit of the
// label dialog is in settings-label-fit.ts. The list shows the Linux labels
// first, then the macOS labels, each from the smallest size to the largest.

import type {
  LabelInput,
  LabelRouting,
  LabelView,
  LabelWaitReason,
  Platform,
} from "../api/schema.ts";
import { labelPlatform, platformName } from "./platform.ts";
import { holdText, providerName } from "./providers.ts";
import { orderText } from "./settings-label-places.ts";

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
};

/** Why the jobs of a label wait, after "Waits: ". */
export function waitText(reason: LabelWaitReason, platform: Platform): string {
  switch (reason.kind) {
    case "empty_order":
      return "no place";
    case "nothing_fits":
      return platform === "macos_arm64" ? "no Mac fits" : "nothing fits";
    case "unavailable":
      return "some facts are not known yet";
    case "provider":
      return holdText(reason.provider, reason.hold);
  }
}

/** Where the jobs of a label run now, as the routing of the API says. */
export function placement(
  routing: LabelRouting,
  platform: Platform,
): Placement {
  switch (routing.kind) {
    case "places":
      return { text: orderText(routing.places, platform), warn: false };
    case "largest":
      return {
        text: `${providerName(routing.provider)}, largest size`,
        warn: false,
      };
    case "waits":
      // A missing fact gives no reason to name, so the row says so plainly.
      return routing.reason.kind === "unavailable"
        ? { text: "Not known yet", warn: false }
        : { text: `Waits: ${waitText(routing.reason, platform)}`, warn: true };
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
      where: placement(view.fit.routing, labelPlatform(view.label)),
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
