// The sizes of the label dialog in words: the size of a label, the VM disk of
// one of its jobs, what the machines offer to its jobs, and the largest Boat
// size of the account. Every number comes from the API fit, formatted to
// whole decimal GB.

import type {
  JobVmSize,
  LabelFit,
  LabelInput,
  LabelMachineFit,
} from "../api/schema.ts";

const nbsp = "\u00a0";
const grouped = new Intl.NumberFormat("en-US", { maximumFractionDigits: 0 });

function unbroken(text: string): string {
  return text.replaceAll(" ", nbsp);
}

/** Bytes as whole decimal GB, to the nearest: "81". */
function wholeGb(bytes: string): string {
  return grouped.format(Math.round(Number(bytes) / 1e9));
}

/** "16 vCPU and 32 GB": the vCPU and the memory of a size. */
function sizePhrase(vcpus: number | string, memoryGb: number | string): string {
  return `${unbroken(`${vcpus} vCPU`)} and ${unbroken(`${memoryGb} GB`)}`;
}

export function labelSizePhrase(label: LabelInput): string {
  return sizePhrase(label.vcpus, label.memory_gb);
}

function vmSizePhrase(size: JobVmSize): string {
  return sizePhrase(size.vcpus, wholeGb(size.memory_bytes));
}

function offerPhrase(machine: LabelMachineFit): string {
  return sizePhrase(machine.vcpus, wholeGb(machine.memory_bytes));
}

/** "With the current image, each VM has a disk of 81 GB.", or null. */
export function vmDiskLine(fit: LabelFit): string | null {
  const root = fit.boat_required_root_disk_bytes ?? null;
  return root === null
    ? null
    : `With the current image, each VM has a disk of ${unbroken(`${wholeGb(root)} GB`)}.`;
}

/**
 * What the machines offer to the jobs of one label, without the sentence
 * end: one machine, the shared offer of several, or the largest machine
 * when their offers differ. Null without an eligible machine.
 */
export function machineOffer(fit: LabelFit): string | null {
  const [first] = fit.machines;
  if (first === undefined) return null;
  if (fit.machines.length === 1) {
    return `Your machine offers ${offerPhrase(first)}`;
  }
  const offers = new Set(fit.machines.map(offerPhrase));
  if (offers.size === 1)
    return `Your machines offer ${offerPhrase(first)} each`;
  const largest = fit.machines.reduce((most, machine) =>
    Number(machine.slots) > Number(most.slots) ? machine : most,
  );
  return `Your largest machine offers ${offerPhrase(largest)}`;
}

export function largestBoat(fit: LabelFit): string | null {
  const largest = fit.largest_boat ?? null;
  return largest === null ? null : vmSizePhrase(largest.size);
}

/** The text of the choice "Run on the largest Boat size". */
export function largestBoatBody(fit: LabelFit | null): string {
  const largest = fit === null ? null : largestBoat(fit);
  return largest === null
    ? "Jobs run on the largest size of your Boat account, also when it is smaller than the label."
    : `Jobs run on ${largest}, the largest size of your Boat account.`;
}
