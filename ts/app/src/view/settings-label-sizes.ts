// The sizes of the label dialog in words: the full request of a label, with
// its vCPU, memory, and free disk, and the VM disk of one of its jobs, from
// the API fit in whole decimal GB. The dialog names the full request of a
// size that nothing fits, never what a machine or a provider size offers.

import type { LabelFit, LabelInput } from "../api/schema.ts";

const nbsp = "\u00a0";
const grouped = new Intl.NumberFormat("en-US", { maximumFractionDigits: 0 });

function unbroken(text: string): string {
  return text.replaceAll(" ", nbsp);
}

/** Bytes as whole decimal GB, to the nearest: "81". */
function wholeGb(bytes: string): string {
  return grouped.format(Math.round(Number(bytes) / 1e9));
}

/** "8 vCPU, 32 GB of memory, and 60 GB of free disk": the full request. */
export function requestPhrase(label: LabelInput): string {
  const vcpus = unbroken(`${label.vcpus} vCPU`);
  const memory = unbroken(`${label.memory_gb} GB`);
  const disk = unbroken(`${label.disk_gb} GB`);
  return `${vcpus}, ${memory} of memory, and ${disk} of free disk`;
}

/**
 * "With the current image, each VM has a disk of 81 GB.", or null. The API
 * gives the complete root of a job of the size with each provider fit, the
 * providers of the order first.
 */
export function vmDiskLine(fit: LabelFit): string | null {
  const root =
    fit.providers.find((provider) => provider.required_root_disk_bytes != null)
      ?.required_root_disk_bytes ?? null;
  return root === null
    ? null
    : `With the current image, each VM has a disk of ${unbroken(`${wholeGb(root)} GB`)}.`;
}
