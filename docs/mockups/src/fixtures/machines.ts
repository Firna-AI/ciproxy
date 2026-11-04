// Own machines of mokly-ai: three Hetzner AX42 machines. A slot holds one job
// VM of 2 vCPU and 8 GB; a larger shape takes several slots.

import { dotJoin, unbroken } from "./format.ts";

export type SlotState = "busy" | "warm" | "free";
export type MachineStatus = "healthy" | "draining" | "offline";

export type Machine = {
  name: string;
  status: MachineStatus;
  slots: readonly SlotState[];
  warmTarget: number;
  cacheUsedGb: number;
  cacheSizeGb: number;
  lastSeen: string;
};

/** The hardware the agent reports; the same for each AX42. */
export const ax42 = {
  cpu: "AMD Ryzen 7 PRO 8700GE",
  threads: 16,
  memoryGb: 64,
  disk: unbroken("2 × 512 GB NVMe"),
  slotsTotal: 6,
  agent: "0.4.2",
  monthlyCost: 53.4,
} as const;

/** "16 threads · 64 GB", the line under a machine name. */
export const ax42Size = dotJoin(
  unbroken(`${ax42.threads} threads`),
  unbroken(`${ax42.memoryGb} GB`),
);

function slots(busy: number, warm: number): SlotState[] {
  const free = ax42.slotsTotal - busy - warm;
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
    status,
    slots: slots(busy, warm),
    warmTarget: 2,
    cacheUsedGb: [182, 167, 154, 12][index - 1] ?? 0,
    cacheSizeGb: 400,
    lastSeen: status === "offline" ? "14m ago" : "3s ago",
  };
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
  /** hetzner-ax42-1 finishes its jobs and takes no new ones. */
  draining: [machine(1, 3, 0, "draining"), machine(2, 6, 0), machine(3, 5, 1)],
  /** A fourth machine joined; Team includes three. */
  fourMachines: [
    machine(1, 5, 1),
    machine(2, 6, 0),
    machine(3, 4, 2),
    machine(4, 0, 2),
  ],
} as const satisfies Record<string, readonly Machine[]>;

export type MachineScene = keyof typeof machineScenes;

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
} as const;
