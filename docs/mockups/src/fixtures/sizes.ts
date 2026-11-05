// VM sizes: the vCPU, memory, and free disk that a label gives each job, and
// the sizes that the Boat account of mokly-ai can start. A Boat size states
// its whole disk, and it fits a label when that disk holds the VM disk of the
// label, the image with the free disk, as vm-disk.ts gives it. The Boat sizes
// are the published small, default, large, and xlarge types; the account is
// on a Boat plan that can start xlarge.

import { dotJoin, unbroken } from "./format.ts";
import { gigabyte, holdsImage, vmDiskBytes } from "./vm-disk.ts";

/** The vCPU, memory, and free disk that a label gives each job. */
export type Size = { vcpu: number; memoryGb: number; freeDiskGb: number };

/** The size of the default label ciproxy. */
export const defaultLabelSize: Size = { vcpu: 2, memoryGb: 8, freeDiskGb: 20 };

/** The least free disk that a label gives each job, in GB. */
export const minimumFreeDiskGb = 1;

/** "8 vCPU · 32 GB · 60 GB free disk", the size of a label. */
export function sizeText(size: Size): string {
  return dotJoin(
    unbroken(`${size.vcpu} vCPU`),
    unbroken(`${size.memoryGb} GB`),
    unbroken(`${size.freeDiskGb} GB free disk`),
  );
}

/** "16 vCPU and 32 GB", the vCPU and memory of a size in a sentence. */
export function sizePhrase(size: Pick<Size, "vcpu" | "memoryGb">): string {
  return `${unbroken(`${size.vcpu} vCPU`)} and ${unbroken(`${size.memoryGb} GB`)}`;
}

/**
 * A size that the Boat account can start: the name of its Boat type, its vCPU
 * and memory, and its whole disk, which also holds the image.
 */
export type BoatSize = {
  name: string;
  vcpu: number;
  memoryGb: number;
  diskGb: number;
};

/** "8 vCPU · 16 GB · 111 GB disk", a Boat size as Boat states it. */
export function boatSizeText(size: BoatSize): string {
  return dotJoin(
    unbroken(`${size.vcpu} vCPU`),
    unbroken(`${size.memoryGb} GB`),
    unbroken(`${size.diskGb} GB disk`),
  );
}

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
 * The largest Boat size runs the jobs of a label that nothing fits, with less
 * free disk than the label when need be, but only when its disk holds the
 * image, as the answer to item 77 says.
 */
export const largestBoatSizeHoldsImage = holdsImage(largestBoatSize.diskGb);

/**
 * The cheapest Boat size with at least the vCPU and memory of a size and a
 * disk that holds its VM disk, which a sandbox for a job of that size gets.
 * It can be larger than the size.
 */
export function boatSizeFor(size: Size): BoatSize | undefined {
  return boatSizes.find(
    (boatSize) =>
      boatSize.vcpu >= size.vcpu &&
      boatSize.memoryGb >= size.memoryGb &&
      boatSize.diskGb * gigabyte >= vmDiskBytes(size),
  );
}

/** A size fits Boat when one Boat size has its vCPU, memory, and VM disk. */
export function fitsBoat(size: Size): boolean {
  return boatSizeFor(size) !== undefined;
}
