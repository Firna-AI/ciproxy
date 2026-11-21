// The machine page: the actions of an Admin for each state, and the notice
// that says why a machine runs no jobs, takes no new ones, or runs fewer jobs
// than its slots. Every value comes from the MachineView of GET machines and
// the overflow summary. The slots are in machines-slots.ts, and the health,
// cache, and cost panels in machines-panels.ts.

import type {
  MachineCommand,
  OverflowSettings,
} from "../api/machine-schema.ts";
import type { MachineView } from "../api/schema.ts";
import type { NoticeTone } from "./page-model.ts";
import { detailCopy, noticeCopy } from "./machines-copy.ts";
import { dotJoin, gigabytes } from "./machines-format.ts";
import { diskLimit } from "./machines-capacity.ts";
import {
  contactSince,
  countRunners,
  hardwareText,
  machineStatus,
  untilReadyText,
} from "./machines.ts";

/**
 * "AMD Ryzen 7 · 16 threads · 64 GB · 1,000 GB disk": the processor, the
 * threads, the memory, and each disk that the agent reported.
 */
export function machineSubtitle(machine: MachineView): string {
  return dotJoin(
    machine.processor,
    hardwareText(machine),
    ...machine.disks.map((disk) =>
      detailCopy.disk(gigabytes(disk.capacity_bytes)),
    ),
  );
}

export type HeaderCommand = Exclude<MachineCommand, "clear-cache">;

export type HeaderAction = {
  command: HeaderCommand;
  label: string;
  danger: boolean;
};

const drain: HeaderAction = {
  command: "drain",
  label: detailCopy.drain,
  danger: false,
};
const resume: HeaderAction = {
  command: "resume",
  label: detailCopy.stopDraining,
  danger: false,
};
const remove: HeaderAction = {
  command: "remove",
  label: detailCopy.remove,
  danger: true,
};

/** The header actions of an Admin, as the mockup gives them for each state. */
export function headerActions(machine: MachineView): HeaderAction[] {
  switch (machineStatus(machine)) {
    case "offline":
      return [{ ...remove, label: detailCopy.removeMachine }];
    case "no-slots":
      return machine.draining ? [resume, remove] : [remove];
    case "draining":
      return [resume];
    case "preparing":
    case "healthy":
      return [drain, remove];
  }
}

export type MachineNotice = {
  key:
    "offline" | "no-kvm" | "no-slots" | "draining" | "preparing" | "disk-limit";
  tone: NoticeTone;
  title: string;
  body: string;
  /** An Admin gets the install command, which is the fix. */
  fix: boolean;
};

function jobsRunning(machine: MachineView): number | null {
  if (machine.occupancy.length === 0) return null;
  return countRunners(machine, "job");
}

/**
 * The notice of a machine that gets its first job image: when jobs start on
 * it, and where they run until then, unless they wait for it.
 */
function preparingNotice(
  machine: MachineView,
  machines: readonly MachineView[],
  summary: OverflowSettings | null,
): MachineNotice {
  const until = untilReadyText(machine, machines, summary);
  return {
    key: "preparing",
    tone: "info",
    title: noticeCopy.preparingTitle(machine.hostname),
    body:
      until === null
        ? noticeCopy.preparing
        : `${noticeCopy.preparing} ${until}`,
    fix: false,
  };
}

function statusNotice(
  machine: MachineView,
  machines: readonly MachineView[],
  summary: OverflowSettings | null,
  now: number,
): MachineNotice | null {
  const host = machine.hostname;
  switch (machineStatus(machine)) {
    case "offline":
      return {
        key: "offline",
        tone: "danger",
        title: noticeCopy.offlineTitle(host),
        body: `${noticeCopy.lostContact(contactSince(machine, now))} ${noticeCopy.offlineCheck}`,
        fix: false,
      };
    case "no-slots":
      return machine.kvm
        ? {
            key: "no-slots",
            tone: "danger",
            title: noticeCopy.noSlotsTitle(host),
            body: noticeCopy.noSlots,
            fix: false,
          }
        : {
            key: "no-kvm",
            tone: "danger",
            title: noticeCopy.noKvmTitle(host),
            body: noticeCopy.noKvm,
            fix: true,
          };
    case "draining":
      return {
        key: "draining",
        tone: "warning",
        title: noticeCopy.drainingTitle,
        body: noticeCopy.draining(jobsRunning(machine)),
        fix: false,
      };
    case "preparing":
      return preparingNotice(machine, machines, summary);
    case "healthy": {
      const limit = diskLimit(machine);
      return limit === null
        ? null
        : {
            key: "disk-limit",
            tone: "warning",
            title: noticeCopy.diskLimitTitle(host, limit.jobs),
            body: noticeCopy.diskLimit(limit.vmDisk, limit.jobDisk),
            fix: false,
          };
    }
  }
}

/**
 * The notice that leads the page: offline, no KVM, no room for a slot, a
 * drain, the first job image, or a disk that holds fewer jobs of ciproxy than
 * the slots, when the API says that the disk sets that number. Each state
 * shows one notice. `machines` and `summary` say where jobs run until a
 * machine has its first job image.
 */
export function machineNotice(
  machine: MachineView,
  machines: readonly MachineView[],
  summary: OverflowSettings | null,
  now: number,
): MachineNotice | null {
  return statusNotice(machine, machines, summary, now);
}
