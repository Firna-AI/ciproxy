// The labels of mokly-ai with their sizes, and its repositories. Each label
// has an order: the places where its jobs run, in order, from label-order.ts.
// In the main story, each Linux label runs on the own machines first and on
// Boat when they are busy. A size that fits no machine runs on Boat only, and
// a size that fits nothing follows the label setting "When nothing fits". The
// disk of a label is the free disk of each of its jobs, and the fit counts
// the whole VM disk of vm-disk.ts. A label keeps its platform, and its jobs
// run only on machines of that platform. A macOS label runs on the Macs only,
// so its jobs wait when no Mac fits them. placement.ts says where the jobs of
// a label run at a moment.

import {
  jobsAtOnce,
  machineSlots,
  slotsFor,
  type Hardware,
} from "./hardware.ts";
import { machinesOnly, machinesThenBoat, type Place } from "./label-order.ts";
import { machineScenes, type Machine } from "./machines.ts";
import { labsOrg, org, type Organization } from "./org.ts";
import type { Platform } from "./platform.ts";
import { defaultLabelSize, defaultMacosLabelSize, type Size } from "./sizes.ts";

/**
 * The label setting "When nothing fits": wait, or run on the largest size of
 * the first available provider of the order whose largest size holds the
 * image.
 */
export type WhenNothingFits = "wait" | "largest-size";

export type Label = Size & {
  name: string;
  /** The platform of its jobs, which never changes. */
  platform: Platform;
  /** Jobs in the last 30 days. */
  jobs: number;
  /** The places where its jobs run, in order. */
  order: readonly Place[];
  whenNothingFits: WhenNothingFits;
};

export const labels: readonly Label[] = [
  {
    name: "ciproxy",
    platform: "linux",
    ...defaultLabelSize,
    jobs: 14_371,
    order: machinesThenBoat,
    whenNothingFits: "wait",
  },
  {
    name: "ciproxy-medium",
    platform: "linux",
    vcpu: 4,
    memoryGb: 16,
    freeDiskGb: 20,
    jobs: 502,
    order: machinesThenBoat,
    whenNothingFits: "wait",
  },
  {
    name: "ciproxy-large",
    platform: "linux",
    vcpu: 8,
    memoryGb: 32,
    freeDiskGb: 60,
    jobs: 2_899,
    order: machinesThenBoat,
    whenNothingFits: "wait",
  },
];

/** A default label by its name. */
export function labelNamed(name: string): Label {
  const label = labels.find((entry) => entry.name === name);
  if (!label) throw new Error(`No label ${name}`);
  return label;
}

/** A new label that no own machine fits, but a Boat size does. */
export const boatOnlyLabel: Label = {
  name: "ciproxy-16vcpu",
  platform: "linux",
  vcpu: 16,
  memoryGb: 32,
  freeDiskGb: 120,
  jobs: 0,
  order: machinesThenBoat,
  whenNothingFits: "wait",
};

/** A new label for load tests that no machine and no Boat size fits. */
export const nothingFitsLabel: Label = {
  name: "ciproxy-32vcpu",
  platform: "linux",
  vcpu: 32,
  memoryGb: 128,
  freeDiskGb: 200,
  jobs: 0,
  order: machinesThenBoat,
  whenNothingFits: "wait",
};

/** A label named in the add dialog that the GitHub label rules refuse. */
export const refusedLabel: Label = {
  name: "load tests",
  platform: "linux",
  vcpu: 4,
  memoryGb: 16,
  freeDiskGb: 100,
  jobs: 0,
  order: machinesThenBoat,
  whenNothingFits: "wait",
};

/** The default macOS label ciproxy-macos. */
export const macosLabel: Label = {
  name: "ciproxy-macos",
  platform: "macos",
  ...defaultMacosLabelSize,
  jobs: 1_204,
  order: machinesOnly,
  whenNothingFits: "wait",
};

/**
 * The default macOS labels, which every workspace has beside the Linux
 * labels. Their jobs run on the Macs of macs.ts in the what-ifs of macOS
 * jobs. The sizes are placeholders until the protocol docs set them.
 */
export const macosLabels: readonly Label[] = [
  macosLabel,
  {
    name: "ciproxy-macos-medium",
    platform: "macos",
    vcpu: 4,
    memoryGb: 16,
    freeDiskGb: 20,
    jobs: 318,
    order: machinesOnly,
    whenNothingFits: "wait",
  },
];

/** A new macOS label for Xcode builds, which the add dialog adds. */
export const largerMacosLabel: Label = {
  name: "ciproxy-macos-xcode",
  platform: "macos",
  vcpu: 8,
  memoryGb: 32,
  freeDiskGb: 60,
  jobs: 0,
  order: machinesOnly,
  whenNothingFits: "wait",
};

/** The machines of the busy afternoon, three Hetzner AX42 machines. */
const steadyMachines = machineScenes.steady;

/**
 * The jobs of a label that a machine runs at a time, by its slots and disk.
 * A machine of another platform runs none.
 */
function machineJobs(machine: Machine, label: Label): number {
  if (machine.hardware.platform !== label.platform) return 0;
  return jobsAtOnce(machine.hardware, machine.slots.length, label);
}

export function fitsOwnMachine(
  label: Label,
  machines: readonly Machine[] = steadyMachines,
): boolean {
  return machines.some((machine) => machineJobs(machine, label) > 0);
}

/**
 * The fewest and the most jobs of this size that one own machine runs at a
 * time, over the machines that fit the size. Machines with different slots
 * or disks give a range, as design call 41 says.
 */
export function jobsPerMachine(
  label: Label,
  machines: readonly Machine[] = steadyMachines,
): { fewest: number; most: number } {
  const counts = machines
    .map((machine) => machineJobs(machine, label))
    .filter((count) => count > 0);
  return { fewest: Math.min(...counts), most: Math.max(...counts) };
}

/** The default label of a platform: ciproxy, or ciproxy-macos on a Mac. */
export function defaultLabelOf(platform: Platform): Label {
  return platform === "macos" ? macosLabel : labelNamed("ciproxy");
}

/**
 * The jobs of the default label of its platform that a machine runs at a
 * time: by its slots alone, and by its slots and its disk. The disk limits
 * the machine when the second number is the smaller.
 */
export function defaultLabelJobs(hardware: Hardware): {
  bySlots: number;
  atOnce: number;
} {
  const label = defaultLabelOf(hardware.platform);
  const slots = machineSlots(hardware);
  return {
    bySlots: Math.floor(slots / slotsFor(label)),
    atOnce: jobsAtOnce(hardware, slots, label),
  };
}

/** Overflow wait times the customer can pick, in seconds. */
export const waitChoices = [0, 15, 30, 60, 120] as const;

/** The repository access of an organization: all private, or a count. */
export type OrganizationAccess = {
  organization: Organization;
  /** The selected repositories, when the access is not all of them. */
  selected?: number;
};

/** mokly-ai allows all its private repositories, mokly-labs three. */
export const organizationAccess: readonly OrganizationAccess[] = [
  { organization: org },
  { organization: labsOrg, selected: 3 },
];

export type Repository = { name: string; selected: boolean };

/** The 14 private repositories that the GitHub App can see. */
export const repositories: readonly Repository[] = [
  { name: "mokly", selected: true },
  { name: "mokly-cloud", selected: true },
  { name: "mokly-site", selected: true },
  { name: "mokly-docs", selected: true },
  { name: "mokly-action", selected: true },
  { name: "viewer-kit", selected: true },
  { name: "design-tokens", selected: false },
  { name: "storybook-adapter", selected: true },
  { name: "examples", selected: false },
  { name: "infra", selected: true },
  { name: "release-tools", selected: true },
  { name: "benchmarks", selected: false },
  { name: "cloud-billing", selected: true },
  { name: "homebrew-tap", selected: false },
];
