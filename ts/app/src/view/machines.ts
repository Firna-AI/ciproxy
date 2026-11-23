// The facts of the Machines list, from GET machines and the overflow summary
// only: the state of each machine, its slots, the figures above the table,
// and where jobs go while a machine takes none. Whether a machine is offline,
// gets its first job image, or is ready comes from the state of the API, which
// routing reads too. A fact that the API does not give stays out, or shows as
// not reported. machines-cache.ts gives the cache figures.

import type { MachineView, SlotState, SlotView } from "../api/schema.ts";
import type { OverflowSettings } from "../api/machine-schema.ts";
import { boatProvider } from "./boat.ts";
import { cacheHits, type CacheHits } from "./machines-cache.ts";
import {
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
import { canStartVms, isMac, platformName } from "./platform.ts";

export type MachineStatus = keyof typeof statusWords;

/**
 * Offline first, then a machine that cannot run jobs, then a drain, then a
 * machine that gets its first job image. A machine that cannot start job VMs
 * gets no slots, as the agent doc says: a Linux machine without KVM, or a Mac
 * without the virtualization that its job image needs. A drained machine takes no new job either way, and
 * the drain is the customer's own action, so it shows over the image wait.
 */
export function machineStatus(machine: MachineView): MachineStatus {
  if (machine.state === "offline") return "offline";
  if (!canStartVms(machine) || machine.slots === 0) return "no-slots";
  if (machine.draining) return "draining";
  if (machine.state === "preparing") return "preparing";
  return "healthy";
}

/**
 * Whether the slots of the machine run jobs or can take them, so that the
 * figures and the jobs at a time count them: the API says that the machine
 * is ready, and it has slots. A drain still runs its jobs. An offline machine
 * reports old slots, and a machine that gets its first job image runs no job,
 * also while it drains.
 */
export function slotsLive(machine: MachineView): boolean {
  const status = machineStatus(machine);
  return (
    machine.state === "ready" && (status === "healthy" || status === "draining")
  );
}

/** The reported slots in slot order, each with its own index. */
export function slotEntries(machine: MachineView): SlotView[] {
  return [...machine.occupancy].sort(
    (a, b) => Number(a.index) - Number(b.index),
  );
}

/**
 * The slot bar of the list, in slot order. An offline machine reports old
 * slots, and no job starts in the slots of a machine that gets its first job
 * image, so neither shows a bar. Its caption says why.
 */
export function slotBar(machine: MachineView): SlotState[] {
  const status = machineStatus(machine);
  if (status === "offline" || status === "preparing") return [];
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
  if (status === "preparing") return slotCopy.preparing;
  if (machine.occupancy.length === 0) return slotCopy.unreported(machine.slots);
  return slotCopy.busyWarm(
    countState(machine, "job"),
    countState(machine, "warm"),
  );
}

/**
 * "Linux · 16 threads · 64 GB", the line beside a machine name. Apple Silicon
 * runs one thread on each core, so a Mac names its cores.
 */
export function hardwareText(machine: MachineView): string {
  return dotJoin(platformName[machine.platform], machineSize(machine));
}

/** "16 threads · 64 GB", or "16 cores · 64 GB" on a Mac. */
export function machineSize(machine: MachineView): string {
  const unit = isMac(machine) ? "cores" : "threads";
  return dotJoin(
    unbroken(`${machine.threads} ${unit}`),
    unbroken(`${gigabytes(machine.memory_bytes)} GB`),
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

export type MachineFigures = {
  machines: string;
  slots: string;
  warm: string;
  cacheHits: CacheHits;
};

/**
 * The figures above the table. Slots and warm runners count only the machines
 * whose slots are live: the occupancy of an offline machine is old, and a
 * machine that gets its first job image takes no job yet.
 */
export function machineFigures(
  machines: readonly MachineView[],
): MachineFigures {
  const live = machines.filter(slotsLive);
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
    cacheHits: cacheHits(machines),
  };
}

/** Whether Boat takes new jobs now, by the overflow summary. */
export function boatTakesJobs(summary: OverflowSettings | null): boolean {
  if (summary === null) return false;
  const provider = boatProvider(summary);
  const account = provider?.account ?? null;
  return (
    provider?.available === true &&
    summary.enabled &&
    account !== null &&
    (account.rejected_at ?? null) === null &&
    provider.spend?.limit_reached !== true
  );
}

/** Whether a machine other than `machine` takes new jobs now. */
export function otherMachineTakesJobs(
  machine: MachineView,
  machines: readonly MachineView[],
): boolean {
  return machines.some(
    (other) => other.id !== machine.id && machineStatus(other) === "healthy",
  );
}

/**
 * The sentence of where jobs go while `machine` takes none: the other
 * machines that take jobs now, and Boat when it takes jobs. Null when no
 * other place takes them.
 */
function elsewhereSentence(
  machine: MachineView,
  machines: readonly MachineView[],
  summary: OverflowSettings | null,
): ((subject: string) => string) | null {
  const others = otherMachineTakesJobs(machine, machines);
  const boat = boatTakesJobs(summary);
  if (others && boat) return elsewhereCopy.machinesAndBoat;
  if (others) return elsewhereCopy.machines;
  if (boat) return elsewhereCopy.boat;
  return null;
}

/** Where jobs go while `machine` takes none. `subject` leads the sentence. */
export function elsewhereText(
  machine: MachineView,
  machines: readonly MachineView[],
  summary: OverflowSettings | null,
  subject: string = elsewhereCopy.jobs,
): string {
  const sentence = elsewhereSentence(machine, machines, summary);
  return (sentence ?? elsewhereCopy.wait)(subject);
}

/**
 * Where jobs run until a machine that gets its first job image is ready, or
 * null when no other place takes them, so they wait for it.
 */
export function untilReadyText(
  machine: MachineView,
  machines: readonly MachineView[],
  summary: OverflowSettings | null,
): string | null {
  const sentence = elsewhereSentence(machine, machines, summary);
  return sentence === null ? null : sentence(elsewhereCopy.untilReady);
}
