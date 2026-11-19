// A what-if of the busy afternoon of 18 November: hetzner-ax42-4, a fourth
// AX42, joined minutes ago and gets its first job image. Until the image is
// ready, the machine takes no job, so no VM starts in its slots, and its
// cache served no restore. Its slots do not count on Machines. Team includes
// three machines, and a machine counts from the moment it registers.

import { ax42, jobDiskBytes, machineSlots } from "./hardware.ts";
import {
  defaultWarmRunners,
  latestAgentVersion,
  machineScenes,
  type Machine,
  type SlotState,
} from "./machines.ts";
import { gigabyte } from "./vm-disk.ts";

/** hetzner-ax42-4 while it gets its first job image. */
export const preparingAx42: Machine = {
  name: "hetzner-ax42-4",
  hardware: ax42,
  status: "preparing",
  slots: Array<SlotState>(machineSlots(ax42)).fill("free"),
  kvm: true,
  warmTarget: defaultWarmRunners,
  cache: {
    status: "measured",
    usedGb: 0,
    sizeGb: Math.round(jobDiskBytes(ax42) / gigabyte),
    noRestores: true,
  },
  freeDiskGb: 958,
  dockerDisks: { status: "measured", repositories: 0, sizeGb: 0 },
  lastSeen: "3s ago",
  agentVersion: latestAgentVersion,
};

/** The machines of the busy afternoon, with hetzner-ax42-4 as the fourth. */
export const newMachineScene: readonly Machine[] = [
  ...machineScenes.steady,
  preparingAx42,
];
