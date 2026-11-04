// VM sizes: the vCPU, memory, and disk that a label gives a job, and the
// sizes that the Boat account of mokly-ai can start. The Boat sizes are the
// published small, default, large, and xlarge types; the account is on a Boat
// plan that can start xlarge.

import { dotJoin, unbroken } from "./format.ts";

export type Size = { vcpu: number; memoryGb: number; diskGb: number };

/** "8 vCPU · 32 GB · 120 GB disk", the size of a label. */
export function sizeText(size: Size): string {
  return dotJoin(
    unbroken(`${size.vcpu} vCPU`),
    unbroken(`${size.memoryGb} GB`),
    unbroken(`${size.diskGb} GB disk`),
  );
}

/** "16 vCPU and 32 GB", the vCPU and memory of a size in a sentence. */
export function sizePhrase(size: Pick<Size, "vcpu" | "memoryGb">): string {
  return `${unbroken(`${size.vcpu} vCPU`)} and ${unbroken(`${size.memoryGb} GB`)}`;
}

/** The sizes that the Boat account can start now, smallest first. */
export const boatSizes: readonly Size[] = [
  { vcpu: 2, memoryGb: 4, diskGb: 51 },
  { vcpu: 4, memoryGb: 8, diskGb: 51 },
  { vcpu: 8, memoryGb: 16, diskGb: 111 },
  { vcpu: 16, memoryGb: 32, diskGb: 251 },
];

export const largestBoatSize: Size = boatSizes.reduce((largest, size) =>
  size.vcpu > largest.vcpu ? size : largest,
);

/** A size fits Boat when one Boat size has at least its vCPU, memory, and disk. */
export function fitsBoat(size: Size): boolean {
  return boatSizes.some(
    (boatSize) =>
      boatSize.vcpu >= size.vcpu &&
      boatSize.memoryGb >= size.memoryGb &&
      boatSize.diskGb >= size.diskGb,
  );
}
