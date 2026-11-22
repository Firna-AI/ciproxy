// The labels of mokly-ai with their sizes, and its repositories. A label
// runs on the own machines first and on Boat when they are busy. A size that
// fits no machine runs on Boat only, and a size that fits nothing follows the
// label setting "When nothing fits". The disk of a label is the free disk of
// each of its jobs, and the fit counts the whole VM disk of vm-disk.ts. A
// label keeps its platform, and its jobs run only on machines of that
// platform. A macOS label runs on the Macs only and never on Boat, so its
// jobs wait when no Mac fits them.

import {
  jobsAtOnce,
  machineSlots,
  slotsFor,
  type Hardware,
} from "./hardware.ts";
import { machineScenes, type Machine } from "./machines.ts";
import { labsOrg, org, type Organization } from "./org.ts";
import type { Platform } from "./platform.ts";
import {
  defaultLabelSize,
  defaultMacosLabelSize,
  fitsBoat,
  largestBoatSizeHoldsImage,
  type Size,
} from "./sizes.ts";

/** The label setting "When nothing fits". */
export type WhenNothingFits = "wait" | "largest-boat-size";

export type Label = Size & {
  name: string;
  /** The platform of its jobs, which never changes. */
  platform: Platform;
  /** Jobs in the last 30 days. */
  jobs: number;
  whenNothingFits: WhenNothingFits;
};

export const labels: readonly Label[] = [
  {
    name: "ciproxy",
    platform: "linux",
    ...defaultLabelSize,
    jobs: 14_371,
    whenNothingFits: "wait",
  },
  {
    name: "ciproxy-4vcpu",
    platform: "linux",
    vcpu: 4,
    memoryGb: 16,
    freeDiskGb: 20,
    jobs: 502,
    whenNothingFits: "wait",
  },
  {
    name: "ciproxy-8vcpu",
    platform: "linux",
    vcpu: 8,
    memoryGb: 32,
    freeDiskGb: 60,
    jobs: 2_899,
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
  whenNothingFits: "wait",
};

/** The default macOS label ciproxy-macos. */
export const macosLabel: Label = {
  name: "ciproxy-macos",
  platform: "macos",
  ...defaultMacosLabelSize,
  jobs: 1_204,
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
    name: "ciproxy-macos-4vcpu",
    platform: "macos",
    vcpu: 4,
    memoryGb: 16,
    freeDiskGb: 20,
    jobs: 318,
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

/** Where the jobs of a label run now. */
export type Placement =
  "machines-then-boat" | "machines-only" | "macs-only" | "boat-only" | "waits";

/**
 * Where the jobs of a label run now. Without a Boat account, as after a setup
 * that skipped Boat, they run on the machines only, and a size that fits no
 * machine waits; no Boat size is checked.
 */
export function placementOf(
  label: Label,
  boatConnected = true,
  machines: readonly Machine[] = steadyMachines,
): Placement {
  if (label.platform === "macos") {
    return fitsOwnMachine(label, machines) ? "macs-only" : "waits";
  }
  if (!boatConnected) return fitsOwnMachine(label) ? "machines-only" : "waits";
  if (fitsOwnMachine(label)) return "machines-then-boat";
  if (fitsBoat(label)) return "boat-only";
  if (
    label.whenNothingFits === "largest-boat-size" &&
    largestBoatSizeHoldsImage
  )
    return "boat-only";
  return "waits";
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
