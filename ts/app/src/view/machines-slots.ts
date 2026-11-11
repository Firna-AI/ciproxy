// The slots of the machine page: each reported slot in index order with its
// job, its warm runner, or nothing, and the size of a slot. A busy slot names
// the dashboard job that the API matched to it, with the name of its
// repository, its pull request or branch, how long it has run, and the job
// identity for its link. A report job that the API could not match names no
// job and links nowhere. A free slot that the disk cannot serve says so.

import type { OverflowSettings } from "../api/machine-schema.ts";
import type {
  JobView,
  MachineView,
  SlotState,
  SlotView,
} from "../api/schema.ts";
import { slotCopy } from "./machines-copy.ts";
import { formatRuntime } from "./machines-format.ts";
import { concurrentJobs } from "./machines-capacity.ts";
import { elsewhereText, machineStatus, slotEntries } from "./machines.ts";

/** The dashboard job of a busy slot. `id` is its canonical identity. */
export type SlotJob = {
  id: string;
  name: string;
  context: string;
  /** "4m 12s", or null when the API does not know when it started. */
  duration: string | null;
};

export type SlotRow = {
  key: string;
  label: string;
  state: SlotState;
  text: string;
  job: SlotJob | null;
};

export type SlotsModel =
  | { kind: "message"; text: string }
  | {
      kind: "slots";
      states: SlotState[];
      rows: SlotRow[];
      caption: string;
      /** The jobs of ciproxy that run at a time, when the API knows it. */
      fits: string | null;
    };

const pullRef = /^refs\/pull\/(\d+)\/(?:merge|head)$/;

/** "#1542" for a pull request, else the branch or tag, or null. */
function refName(job: JobView): string | null {
  const ref = job.git_ref ?? null;
  const pull = ref === null ? null : pullRef.exec(ref);
  if (pull) return `#${pull[1]}`;
  const branch = job.head_branch ?? null;
  if (branch !== null) return branch;
  if (ref === null) return null;
  return ref.replace(/^refs\/(?:heads|tags)\//, "");
}

/** "mokly #1542": the repository name, with its pull request or branch. */
export function jobContext(job: JobView): string {
  const name = job.repository.slice(job.repository.lastIndexOf("/") + 1);
  const ref = refName(job);
  return ref === null ? name : `${name} ${ref}`;
}

function slotJob(slot: SlotView): SlotJob | null {
  const job = slot.job ?? null;
  if (slot.state !== "job" || job === null) return null;
  const seconds = job.runtime_seconds ?? null;
  return {
    id: job.id,
    name: job.name,
    context: jobContext(job),
    duration: seconds === null ? null : formatRuntime(seconds),
  };
}

/**
 * How many free slots the disk cannot serve: the slots beyond the jobs of
 * ciproxy that its disk holds. The API counts both the jobs by slots and by
 * disk; when a job of ciproxy takes one slot, their difference is a number of
 * slots. Another size of ciproxy leaves every free slot plain, and the
 * notice of the page still names the limit.
 */
function slotsWithoutDisk(machine: MachineView): number {
  const byDisk = machine.concurrent_default_jobs ?? null;
  const bySlots = machine.default_jobs_by_slots ?? null;
  if (machine.disk_limited !== true || byDisk === null || bySlots === null) {
    return 0;
  }
  if (Number(bySlots) !== machine.slots) return 0;
  return Math.max(0, machine.slots - Number(byDisk));
}

function slotRows(machine: MachineView, entries: SlotView[]): SlotRow[] {
  const free = entries.filter((slot) => slot.state === "free");
  const without = Math.min(free.length, slotsWithoutDisk(machine));
  const noDisk = new Set(
    free.slice(free.length - without).map((slot) => slot.index),
  );
  return entries.map((slot) => {
    const job = slotJob(slot);
    const text =
      slot.state === "job"
        ? (job?.name ?? slotCopy.job)
        : slot.state === "warm"
          ? slotCopy.warm
          : noDisk.has(slot.index)
            ? slotCopy.freeNoDisk
            : slotCopy.free;
    return {
      key: slot.index,
      label: slotCopy.row(Number(slot.index) + 1),
      state: slot.state,
      text,
      job,
    };
  });
}

export function slotsModel(
  machine: MachineView,
  machines: readonly MachineView[],
  summary: OverflowSettings | null,
): SlotsModel {
  const status = machineStatus(machine);
  if (status === "offline")
    return { kind: "message", text: slotCopy.offlineBody };
  if (status === "no-slots") {
    const elsewhere = elsewhereText(machine, machines, summary);
    return { kind: "message", text: slotCopy.noSlotsBody(elsewhere) };
  }
  const entries = slotEntries(machine);
  if (entries.length === 0) {
    return { kind: "message", text: slotCopy.unreportedBody };
  }
  const jobs = concurrentJobs(machine);
  return {
    kind: "slots",
    states: entries.map((slot) => slot.state),
    rows: slotRows(machine, entries),
    caption: slotCopy.size(machine.slots),
    fits: jobs === null ? null : slotCopy.fits(jobs),
  };
}
