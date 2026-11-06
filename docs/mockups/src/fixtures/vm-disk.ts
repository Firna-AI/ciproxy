// The VM disk of a job. As the answer to item 77 says, the disk of a label is
// the free disk that each of its jobs gets. The VM disk holds the root disk of
// the current image release, grown by that free disk and the reserve pool
// that the kernel keeps on ext4, with an allowance of 3% of the growth for the
// root reserve and the new filesystem metadata, in whole blocks of 4,096
// bytes. RootSizing in ciproxy-agent holds the rule that the agent and the
// scheduler share, and this file mirrors it. An own machine reserves the VM
// disk of each job VM, and a Boat size fits a label when its disk holds that
// VM disk. Disks use decimal GB, and the rule whole bytes.

import { formatCount, unbroken } from "./format.ts";

/** One decimal GB, in bytes. */
export const gigabyte = 1_000_000_000;

/** The measured root disk of the current image release, in bytes. */
export const currentImageRootBytes = 60_263_759_872;

/** The most that the kernel keeps back on ext4: 4,096 blocks of 4,096 bytes. */
const kernelReserveBytes = 16 * 1_024 * 1_024;

/** The VM disk grows in whole blocks of this many bytes. */
const blockBytes = 4_096;

/** a / b rounded down, exact for whole numbers below 2^53. */
function floorDiv(a: number, b: number): number {
  return (a - (a % b)) / b;
}

/** a / b rounded up, exact for whole numbers below 2^53. */
function ceilDiv(a: number, b: number): number {
  return floorDiv(a, b) + (a % b === 0 ? 0 : 1);
}

/** The VM disk of a job of a size with the current image, in bytes. */
export function vmDiskBytes(size: { freeDiskGb: number }): number {
  const freeBytes = size.freeDiskGb * gigabyte + kernelReserveBytes;
  const growth = ceilDiv(freeBytes * 100, 97);
  return ceilDiv(currentImageRootBytes + growth, blockBytes) * blockBytes;
}

/** How many jobs of a size a disk of this many bytes holds at a time. */
export function jobsInDisk(
  diskBytes: number,
  size: { freeDiskGb: number },
): number {
  return floorDiv(diskBytes, vmDiskBytes(size));
}

/** A disk of this many GB holds the current image. */
export function holdsImage(diskGb: number): boolean {
  return diskGb * gigabyte >= currentImageRootBytes;
}

/** Bytes as whole GB, to the nearest: 80,899,612,672 bytes are "81 GB". */
export function diskText(bytes: number): string {
  return unbroken(`${formatCount(Math.round(bytes / gigabyte))} GB`);
}
