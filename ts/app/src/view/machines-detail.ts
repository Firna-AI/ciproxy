// The machine page: the actions of an Admin for each state, the notice that
// says why a machine runs no jobs or takes no new ones, and its slots. Every
// value comes from the MachineView of GET machines. The health, cache, and
// cost panels are in machines-panels.ts.

import type {
  MachineCommand,
  OverflowSettings,
} from "../api/machine-schema.ts";
import type { MachineView, SlotState } from "../api/schema.ts";
import type { NoticeTone } from "./page-model.ts";
import { detailCopy, noticeCopy, slotCopy } from "./machines-copy.ts";
import { dotJoin, gigabytes } from "./machines-format.ts";
import {
  concurrentJobs,
  contactSince,
  countRunners,
  elsewhereText,
  hardwareText,
  machineStatus,
  slotEntries,
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
    case "healthy":
      return [drain, remove];
  }
}

export type MachineNotice = {
  key: "offline" | "no-kvm" | "no-slots" | "draining";
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

function statusNotice(machine: MachineView, now: number): MachineNotice | null {
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
    case "healthy":
      return null;
  }
}

/**
 * The notice that leads the page: offline, no KVM, no room for a slot, or a
 * drain. A machine whose disk holds fewer jobs than its slots gets no notice
 * yet, because the API does not say whether the disk or the slots set its
 * number of jobs; the slots panel shows that number.
 */
export function machineNotice(
  machine: MachineView,
  now: number,
): MachineNotice | null {
  return statusNotice(machine, now);
}

export type SlotRow = {
  key: string;
  label: string;
  state: SlotState;
  text: string;
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

const slotText: Readonly<Record<SlotState, string>> = {
  job: slotCopy.job,
  warm: slotCopy.warm,
  free: slotCopy.free,
};

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
    rows: entries.map((slot) => ({
      key: slot.index,
      label: slotCopy.row(Number(slot.index) + 1),
      state: slot.state,
      text: slotText[slot.state],
    })),
    caption: slotCopy.size(machine.slots),
    fits: jobs === null ? null : slotCopy.fits(jobs),
  };
}
