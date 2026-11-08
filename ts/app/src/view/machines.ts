// The facts of the Machines list, from GET machines and the overflow summary
// only: the state of each machine, its slots and cache, the figures above the
// table, and where jobs go while a machine takes none. A fact that the API
// does not give stays out, or shows as not reported.

import type { MachineView, SlotState, SlotView } from "../api/schema.ts";
import type { OverflowSettings } from "../api/machine-schema.ts";
import {
  cacheCopy,
  elsewhereCopy,
  healthCopy,
  listCopy,
  slotCopy,
  statusWords,
} from "./machines-copy.ts";
import {
  dotJoin,
  formatAgo,
  formatCount,
  formatSince,
  gigabytes,
  unbroken,
} from "./machines-format.ts";

export type MachineStatus = keyof typeof statusWords;

/**
 * Offline first, then a machine that cannot run jobs, then a drain. A machine
 * without KVM gets no slots, as the agent doc says.
 */
export function machineStatus(machine: MachineView): MachineStatus {
  if (machine.offline) return "offline";
  if (!machine.kvm || machine.slots === 0) return "no-slots";
  if (machine.draining) return "draining";
  return "healthy";
}

/** The reported slots in slot order, each with its own index. */
export function slotEntries(machine: MachineView): SlotView[] {
  return [...machine.occupancy].sort(
    (a, b) => Number(a.index) - Number(b.index),
  );
}

export function slotStates(machine: MachineView): SlotState[] {
  return slotEntries(machine).map((slot) => slot.state);
}

function countState(machine: MachineView, state: SlotState): number {
  return machine.occupancy.filter((slot) => slot.state === state).length;
}

/**
 * The runners in the slots of one state. A VM of a larger size takes several
 * slots, so one job or warm runner counts once, by its job or runner ID.
 */
export function countRunners(machine: MachineView, state: SlotState): number {
  const seen = new Set<string>();
  for (const slot of machine.occupancy) {
    if (slot.state !== state) continue;
    seen.add(slot.job_id ?? slot.runner_id ?? `slot:${slot.index}`);
  }
  return seen.size;
}

/** "5 busy · 1 warm", or why the machine shows no slots. */
export function slotCaption(machine: MachineView): string {
  const status = machineStatus(machine);
  if (status === "offline") return slotCopy.offline;
  if (status === "no-slots") return slotCopy.none;
  if (machine.occupancy.length === 0) return slotCopy.unreported(machine.slots);
  return slotCopy.busyWarm(
    countState(machine, "job"),
    countState(machine, "warm"),
  );
}

/** "16 threads · 64 GB", the line beside a machine name. */
export function hardwareText(machine: MachineView): string {
  return dotJoin(
    unbroken(`${machine.threads} threads`),
    unbroken(`${gigabytes(machine.memory_bytes)} GB`),
  );
}

export function cacheText(machine: MachineView): string {
  const cache = machine.cache ?? null;
  if (cache === null) return cacheCopy.noReport;
  return cacheCopy.used(
    gigabytes(cache.used_bytes),
    gigabytes(cache.capacity_bytes),
  );
}

/** "14 minutes": the time since the last contact, or null without one. */
export function contactSince(machine: MachineView, now: number): string | null {
  const contact = machine.last_contact_at ?? null;
  return contact === null ? null : formatSince(contact, now);
}

export function lastSeenText(machine: MachineView, now: number): string {
  const contact = machine.last_contact_at ?? null;
  return contact === null ? healthCopy.noContact : formatAgo(contact, now);
}

/**
 * The share of the cache restores of the last 7 days that the machine cache
 * served, rounded down so it never overstates, or the words for none.
 */
export function cacheHitsText(
  machines: readonly Pick<MachineView, "cache">[],
): string {
  let hits = BigInt(0);
  let restores = BigInt(0);
  for (const machine of machines) {
    if (!machine.cache) continue;
    hits += BigInt(machine.cache.hits_last_7_days);
    restores += BigInt(machine.cache.restores_last_7_days);
  }
  if (restores <= BigInt(0)) return listCopy.noRestores;
  return `${(hits * BigInt(100)) / restores}%`;
}

export type MachineFigures = {
  machines: string;
  slots: string;
  warm: string;
  cacheHits: string;
};

/**
 * The figures above the table. Slots count only the machines that can run
 * jobs now; the occupancy of an offline machine is old, so it counts none.
 */
export function machineFigures(
  machines: readonly MachineView[],
): MachineFigures {
  const live = machines.filter((machine) => {
    const status = machineStatus(machine);
    return status === "healthy" || status === "draining";
  });
  const total = live.reduce((sum, machine) => sum + machine.slots, 0);
  const used = live.reduce(
    (sum, machine) => sum + countState(machine, "job"),
    0,
  );
  const warm = live.reduce(
    (sum, machine) => sum + countRunners(machine, "warm"),
    0,
  );
  return {
    machines: formatCount(machines.length),
    slots: listCopy.slotsOf(used, total),
    warm: formatCount(warm),
    cacheHits: cacheHitsText(machines),
  };
}

/** Whether Boat takes new jobs now, by the overflow summary. */
export function boatTakesJobs(summary: OverflowSettings | null): boolean {
  if (summary === null) return false;
  const account = summary.account ?? null;
  return (
    summary.available &&
    summary.enabled &&
    account !== null &&
    (account.rejected_at ?? null) === null &&
    summary.spend?.limit_reached !== true
  );
}

/**
 * Where jobs go while `machine` takes none: the other machines that run jobs
 * now, and Boat when it takes jobs. `subject` leads the sentence.
 */
export function elsewhereText(
  machine: MachineView,
  machines: readonly MachineView[],
  summary: OverflowSettings | null,
  subject: string = elsewhereCopy.jobs,
): string {
  const others = machines.some(
    (other) => other.id !== machine.id && machineStatus(other) === "healthy",
  );
  const boat = boatTakesJobs(summary);
  if (others && boat) return elsewhereCopy.machinesAndBoat(subject);
  if (others) return elsewhereCopy.machines(subject);
  if (boat) return elsewhereCopy.boat(subject);
  return elsewhereCopy.wait(subject);
}
