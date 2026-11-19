// Own machines of mokly-ai: three Hetzner AX42 machines, and in some scenes a
// fourth: the mini PC office-mini-1, whose disk holds fewer jobs than its
// slots, or the office PC office-pc-1, whose memory leaves no room for a
// slot. A slot holds one job VM of the smallest size; a larger VM size takes
// several slots. A machine without KVM connects with no slots, so it cannot
// run jobs. In some what-ifs, a machine did not measure its machine cache or
// its Docker disks, so that part of its report has no figures.

import { unbroken } from "./format.ts";
import {
  ax42,
  jobDiskBytes,
  machineSlots,
  miniPc,
  oldPc,
  type Hardware,
} from "./hardware.ts";
import { defaultLabelSize } from "./sizes.ts";
import { gigabyte, vmDiskBytes } from "./vm-disk.ts";

export type SlotState = "busy" | "warm" | "free";
/**
 * The state of a machine. A preparing machine is online and gets its first
 * job image, so it takes no job until the image is ready.
 */
export type MachineStatus =
  "healthy" | "draining" | "offline" | "no-slots" | "preparing";

/**
 * The figures of one part of a machine report: the figures that the machine
 * measured, or unavailable when it did not measure them. An unavailable part
 * has no figures, so no screen shows a zero or a share in their place.
 */
export type Statistics<T> =
  ({ status: "measured" } & T) | { status: "unavailable" };

/** A part of a machine report that the machine did not measure. */
const unavailable = { status: "unavailable" } as const;

export type Machine = {
  name: string;
  hardware: Hardware;
  status: MachineStatus;
  slots: readonly SlotState[];
  /** The machine can start a VM for each job. */
  kvm: boolean;
  /**
   * The saved warm runner count, which an Admin sets on the machine page. It
   * stays while the machine drains, is offline, or has no slots. The warm
   * slots are the warm runners that are ready.
   */
  warmTarget: number;
  /**
   * The machine cache: its use, and its disk now, the job disk that no job
   * VM holds. Its cache hits are `cacheHitsPercent` while it is measured,
   * unless it served no restore yet, as the cache of a new machine.
   */
  cache: Statistics<{ usedGb: number; sizeGb: number; noRestores?: true }>;
  /**
   * The free disk that the agent reported last: the space that no data uses.
   * It can be more than the free job disk, because a job VM uses only the
   * space that it writes.
   */
  freeDiskGb: number;
  /** The repositories with a Docker disk on the machine, and their space. */
  dockerDisks: Statistics<{ repositories: number; sizeGb: number }>;
  lastSeen: string;
  /** The agent release that the machine reported last. */
  agentVersion: string;
};

/** The warm runners that each machine keeps unless the customer changes it. */
export const defaultWarmRunners = 2;

/** The newest agent release, which every machine runs unless a what-if says. */
export const latestAgentVersion = "0.4.2";

/**
 * The job disk that the job VMs leave to the machine cache, which shares it
 * with the Docker disks, in whole GB. Each busy or warm slot of a scene holds
 * a job VM of the label ciproxy, with its VM disk.
 */
function cacheSpaceGb(hardware: Hardware, jobVms: number): number {
  const vmDisks = jobVms * vmDiskBytes(defaultLabelSize);
  return Math.round((jobDiskBytes(hardware) - vmDisks) / gigabyte);
}

function slots(total: number, busy: number, warm: number): SlotState[] {
  const free = total - busy - warm;
  return [
    ...Array<SlotState>(busy).fill("busy"),
    ...Array<SlotState>(warm).fill("warm"),
    ...Array<SlotState>(free).fill("free"),
  ];
}

function machine(
  index: number,
  busy: number,
  warm: number,
  status: MachineStatus = "healthy",
): Machine {
  return {
    name: `hetzner-ax42-${index}`,
    hardware: ax42,
    status,
    slots: slots(machineSlots(ax42), busy, warm),
    kvm: true,
    warmTarget: defaultWarmRunners,
    cache: {
      status: "measured",
      usedGb: [83, 76, 70][index - 1] ?? 0,
      sizeGb: cacheSpaceGb(ax42, busy + warm),
    },
    freeDiskGb: 312,
    dockerDisks: {
      status: "measured",
      repositories: [3, 2, 2][index - 1] ?? 0,
      sizeGb: [96, 58, 61][index - 1] ?? 0,
    },
    lastSeen: status === "offline" ? "14m ago" : "3s ago",
    agentVersion: latestAgentVersion,
  };
}

/**
 * office-mini-1 joined on 18 November: two warm runners wait in its slots.
 * Their VM disks hold 162 GB of the 279 GB that jobs can use, and the cache
 * has the other 117 GB.
 */
export const officeMini: Machine = {
  name: "office-mini-1",
  hardware: miniPc,
  status: "healthy",
  slots: slots(machineSlots(miniPc), 0, 2),
  kvm: true,
  warmTarget: defaultWarmRunners,
  cache: { status: "measured", usedGb: 6, sizeGb: cacheSpaceGb(miniPc, 2) },
  freeDiskGb: 362,
  dockerDisks: { status: "measured", repositories: 0, sizeGb: 0 },
  lastSeen: "3s ago",
  agentVersion: latestAgentVersion,
};

/** hetzner-ax42-3 after a firmware update turned virtualization off. */
const withoutKvm: Machine = {
  ...machine(3, 0, 0),
  status: "no-slots",
  slots: [],
  kvm: false,
};

/**
 * office-pc-1, an older office PC that joined on 18 November. KVM works, but
 * its memory leaves no room for one slot, so it connects with no slots.
 */
export const officePc: Machine = {
  name: "office-pc-1",
  hardware: oldPc,
  status: "no-slots",
  slots: [],
  kvm: true,
  warmTarget: defaultWarmRunners,
  cache: { status: "measured", usedGb: 0, sizeGb: cacheSpaceGb(oldPc, 0) },
  freeDiskGb: 183,
  dockerDisks: { status: "measured", repositories: 0, sizeGb: 0 },
  lastSeen: "3s ago",
  agentVersion: latestAgentVersion,
};

/** hetzner-ax42-1 while its agent still reports the release before 0.4.2. */
const agentBehindAx42: Machine = { ...machine(1, 5, 1), agentVersion: "0.4.1" };

/** The machine, when it did not measure its machine cache for its report. */
function cacheUnmeasured(of: Machine): Machine {
  return { ...of, cache: unavailable };
}

/** The machine, when it did not measure its Docker disks for its report. */
function dockerUnmeasured(of: Machine): Machine {
  return { ...of, dockerDisks: unavailable };
}

/** The moments that the machine screens show. */
export const machineScenes = {
  /** Wednesday 18 November, 14:32: a busy afternoon. */
  steady: [machine(1, 5, 1), machine(2, 6, 0), machine(3, 4, 2)],
  /** Wednesday 18 November, 10:05: every own slot is busy. */
  peak: [machine(1, 6, 0), machine(2, 6, 0), machine(3, 6, 0)],
  /** Sunday 22 November, 03:10: only warm runners wait. */
  quiet: [machine(1, 0, 2), machine(2, 0, 2), machine(3, 0, 2)],
  /** ciproxy lost contact with hetzner-ax42-3 14 minutes ago. */
  offline: [machine(1, 6, 0), machine(2, 5, 1), machine(3, 0, 0, "offline")],
  /** hetzner-ax42-3 connects without KVM, so it gets no slots. */
  noKvm: [machine(1, 5, 1), machine(2, 6, 0), withoutKvm],
  /** hetzner-ax42-1 finishes its jobs and takes no new ones. */
  draining: [machine(1, 3, 0, "draining"), machine(2, 6, 0), machine(3, 5, 1)],
  /**
   * The connection of hetzner-ax42-1 dropped 25 seconds ago, before ciproxy
   * marks it offline, and an Admin drained it.
   */
  drainSaved: [
    { ...machine(1, 3, 0, "draining"), lastSeen: "25s ago" },
    machine(2, 6, 0),
    machine(3, 5, 1),
  ],
  /** office-pc-1 joined as a fourth machine, with no room for a slot. */
  tooSmall: [machine(1, 5, 1), machine(2, 6, 0), machine(3, 4, 2), officePc],
  /**
   * Wednesday 18 November, 14:32: release 0.4.2 is out, and the agent of
   * hetzner-ax42-1 still reports 0.4.1.
   */
  agentBehind: [agentBehindAx42, machine(2, 6, 0), machine(3, 4, 2)],
  /** office-mini-1 joined as a fourth machine; Team includes three. */
  fourMachines: [
    machine(1, 5, 1),
    machine(2, 6, 0),
    machine(3, 4, 2),
    officeMini,
  ],
  /**
   * Wednesday 18 November, 14:32: hetzner-ax42-1 did not measure its machine
   * cache for its last report.
   */
  cacheUnknown: [
    cacheUnmeasured(machine(1, 5, 1)),
    machine(2, 6, 0),
    machine(3, 4, 2),
  ],
  /** The same moment: hetzner-ax42-1 did not measure its Docker disks. */
  dockerUnknown: [
    dockerUnmeasured(machine(1, 5, 1)),
    machine(2, 6, 0),
    machine(3, 4, 2),
  ],
  /** The same moment: hetzner-ax42-1 measured neither part. */
  figuresUnknown: [
    dockerUnmeasured(cacheUnmeasured(machine(1, 5, 1))),
    machine(2, 6, 0),
    machine(3, 4, 2),
  ],
  /** The same moment: no machine measured its machine cache. */
  noCacheMeasured: [
    cacheUnmeasured(machine(1, 5, 1)),
    cacheUnmeasured(machine(2, 6, 0)),
    cacheUnmeasured(machine(3, 4, 2)),
  ],
} as const satisfies Record<string, readonly Machine[]>;

/**
 * The share of its cache restores of the last 7 days that the machine cache
 * served. Each machine that measured its cache has this share, so the
 * machines that Machines counts together have it too.
 */
export const cacheHitsPercent = 91;

/** How long ago ciproxy lost contact with the offline machine. */
export const offlineFor = unbroken("14 minutes");

export function countSlots(
  machines: readonly Machine[],
  state: SlotState,
): number {
  return machines.reduce(
    (sum, m) => sum + m.slots.filter((slot) => slot === state).length,
    0,
  );
}

/** The Boat overflow account of mokly-ai. */
export const boat = {
  account: "mokly",
  spendLimit: 200,
  spentThisMonth: 61.2,
  waitSeconds: 30,
  /** The day Boat stopped accepting the key. */
  keyRejectedOn: unbroken("12 November"),
  /** The UTC month whose Boat spend reached the limit: the budget month. */
  budgetMonth: "November",
  /** The first day of the next month, when Boat takes jobs again. */
  nextMonth: unbroken("1 December"),
} as const;
