// The what-ifs of macOS jobs on 18 November: mokly-ai tests its Safari build
// and builds its macOS app on a Mac of its office, a fourth machine beside the
// three AX42 machines: mac-studio-1, or in a second what-if mac-mini-1. A Mac runs jobs of macOS labels only, each in a fresh macOS VM.
// Its slots follow the rule of a Linux machine, but it runs at most 2 job VMs
// at a time, because Apple allows at most 2 macOS VMs at a time on one Mac.
// mac-studio-1 has 6 slots, so the limit of VMs decides how many jobs it
// runs. The memory of mac-mini-1 leaves room for 1 slot after the host
// reserve. Team includes three machines, and each Mac counts as one machine,
// as a Linux machine does.

import { unbroken } from "./format.ts";
import { jobDiskBytes, machineSlots, type Hardware } from "./hardware.ts";
import {
  latestAgentVersion,
  machineScenes,
  type Machine,
  type SlotState,
} from "./machines.ts";
import { defaultMacosLabelSize } from "./sizes.ts";
import { gigabyte, vmDiskBytes } from "./vm-disk.ts";

/** A Mac Studio with more slots than its 2 macOS VMs use. */
export const macStudio: Hardware = {
  platform: "macos",
  cpu: "Apple M4 Max",
  threads: 16,
  memoryGb: 64,
  disk: unbroken("1 TB SSD"),
  diskGb: 1000,
  monthlyCost: 40,
};

/** A Mac mini whose memory leaves room for 1 slot. */
export const macMini: Hardware = {
  platform: "macos",
  cpu: "Apple M4",
  threads: 10,
  memoryGb: 16,
  disk: unbroken("512 GB SSD"),
  diskGb: 512,
  monthlyCost: 15,
};

/** The warm runners that mokly-ai keeps on each Mac. */
const macWarmRunners = 1;

/** The job disk that the macOS VMs leave to the machine cache, in GB. */
function macCacheGb(hardware: Hardware, jobVms: number): number {
  const vmDisks =
    jobVms * vmDiskBytes({ ...defaultMacosLabelSize, platform: "macos" });
  return Math.round((jobDiskBytes(hardware) - vmDisks) / gigabyte);
}

function macSlots(hardware: Hardware, busy: number): SlotState[] {
  const total = machineSlots(hardware);
  const warm = Math.min(macWarmRunners, total - busy);
  return [
    ...Array<SlotState>(busy).fill("busy"),
    ...Array<SlotState>(warm).fill("warm"),
    ...Array<SlotState>(total - busy - warm).fill("free"),
  ];
}

function mac(name: string, hardware: Hardware, busy: number): Machine {
  const slots = macSlots(hardware, busy);
  return {
    name,
    hardware,
    status: "healthy",
    slots,
    virtualization: true,
    warmTarget: macWarmRunners,
    cache: {
      status: "measured",
      usedGb: 24,
      sizeGb: macCacheGb(hardware, slots.filter((s) => s !== "free").length),
    },
    freeDiskGb: hardware.diskGb === 1000 ? 590 : 232,
    dockerDisks: { status: "measured", repositories: 0, sizeGb: 0 },
    lastSeen: "3s ago",
    agentVersion: latestAgentVersion,
  };
}

/**
 * mac-studio-1: one macOS job and one warm runner, its 2 VMs, so its other 4
 * slots stay free.
 */
export const macStudioMachine = mac("mac-studio-1", macStudio, 1);

/** mac-mini-1: one macOS job in its one slot. */
export const macMiniMachine = mac("mac-mini-1", macMini, 1);

/** The machines of the busy afternoon, with mac-studio-1 as the fourth. */
export const macScene: readonly Machine[] = [
  ...machineScenes.steady,
  macStudioMachine,
];

/**
 * mac-mini-1 when its macOS is older than the macOS of the job image: it
 * cannot start a macOS VM, so it connects with no slots.
 */
export const macMiniOldMacos: Machine = {
  ...macMiniMachine,
  status: "no-slots",
  slots: [],
  virtualization: false,
};
