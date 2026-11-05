// VM sizes: the vCPU, memory, and disk that a label gives a job, and the
// sizes that the Boat account of mokly-ai can start. The Boat sizes are the
// published small, default, large, and xlarge types; the account is on a Boat
// plan that can start xlarge.

import { dotJoin, unbroken } from "./format.ts";

export type Size = { vcpu: number; memoryGb: number; diskGb: number };

/** The size of the default label ciproxy: its disk holds the full image. */
export const defaultLabelSize: Size = { vcpu: 2, memoryGb: 8, diskGb: 80 };

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

/** A size that the Boat account can start, with the name of its Boat type. */
export type BoatSize = Size & { name: string };

/** The sizes that the Boat account can start now, smallest and cheapest first. */
export const boatSizes: readonly BoatSize[] = [
  { name: "small", vcpu: 2, memoryGb: 4, diskGb: 51 },
  { name: "default", vcpu: 4, memoryGb: 8, diskGb: 51 },
  { name: "large", vcpu: 8, memoryGb: 16, diskGb: 111 },
  { name: "xlarge", vcpu: 16, memoryGb: 32, diskGb: 251 },
];

export const largestBoatSize: BoatSize = boatSizes.reduce((largest, size) =>
  size.vcpu > largest.vcpu ? size : largest,
);

/**
 * The cheapest Boat size with at least the vCPU, memory, and disk of a size,
 * which a sandbox for a job of that size gets. It can be larger than the size.
 */
export function boatSizeFor(size: Size): BoatSize | undefined {
  return boatSizes.find(
    (boatSize) =>
      boatSize.vcpu >= size.vcpu &&
      boatSize.memoryGb >= size.memoryGb &&
      boatSize.diskGb >= size.diskGb,
  );
}

/** A size fits Boat when one Boat size has at least its vCPU, memory, and disk. */
export function fitsBoat(size: Size): boolean {
  return boatSizeFor(size) !== undefined;
}
