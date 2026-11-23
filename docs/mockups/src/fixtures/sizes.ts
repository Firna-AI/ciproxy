// VM sizes: the vCPU, memory, and free disk that a label gives each job, and
// the sizes that each provider account of mokly-ai can start. A provider size
// states its whole disk, and it fits a label when that disk holds the VM disk
// of the label, the image with the free disk, as vm-disk.ts gives it. The
// Boat sizes are the published small, default, large, and xlarge types; the
// account is on a Boat plan that can start xlarge. The sizes of Second
// provider are fixture sizes up to 8 vCPU, so the larger labels fit only Boat.

import { dotJoin, unbroken } from "./format.ts";
import type { ProviderKey } from "./providers.ts";
import { diskText, gigabyte, holdsImage, vmDiskBytes } from "./vm-disk.ts";

/** The vCPU, memory, and free disk that a label gives each job. */
export type Size = { vcpu: number; memoryGb: number; freeDiskGb: number };

/** The size of the default label ciproxy. */
export const defaultLabelSize: Size = { vcpu: 2, memoryGb: 8, freeDiskGb: 20 };

/**
 * The size of the default macOS label ciproxy-macos: one slot, as the label
 * ciproxy. A placeholder until the protocol docs set the macOS labels.
 */
export const defaultMacosLabelSize: Size = {
  vcpu: 2,
  memoryGb: 8,
  freeDiskGb: 20,
};

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

/**
 * The VM that ran a job on an own machine, as its runner reported it: its
 * vCPU, its memory, and its VM disk, the image with the free disk.
 */
export type JobVm = { vcpu: number; memoryGb: number; vmDiskBytes: number };

/** "2 vCPU · 8 GB · 81 GB VM disk", the VM that ran a job. */
export function vmSizeText(vm: JobVm): string {
  return dotJoin(
    unbroken(`${vm.vcpu} vCPU`),
    unbroken(`${vm.memoryGb} GB`),
    unbroken(`${diskText(vm.vmDiskBytes)} VM disk`),
  );
}

/** "16 vCPU and 32 GB", the vCPU and memory of a size in a sentence. */
export function sizePhrase(size: Pick<Size, "vcpu" | "memoryGb">): string {
  return `${unbroken(`${size.vcpu} vCPU`)} and ${unbroken(`${size.memoryGb} GB`)}`;
}

/**
 * A size that a provider account can start: the name of its type at the
 * provider, its vCPU and memory, and its whole disk, which also holds the
 * image.
 */
export type ProviderSize = {
  name: string;
  vcpu: number;
  memoryGb: number;
  diskGb: number;
};

/** "8 vCPU · 16 GB · 111 GB disk", a provider size as its provider states it. */
export function providerSizeText(size: ProviderSize): string {
  return dotJoin(
    unbroken(`${size.vcpu} vCPU`),
    unbroken(`${size.memoryGb} GB`),
    unbroken(`${size.diskGb} GB disk`),
  );
}

/**
 * The sizes that each provider account can start now, smallest and cheapest
 * first.
 */
export const providerSizes: Readonly<
  Record<ProviderKey, readonly ProviderSize[]>
> = {
  boat: [
    { name: "small", vcpu: 2, memoryGb: 4, diskGb: 51 },
    { name: "default", vcpu: 4, memoryGb: 8, diskGb: 51 },
    { name: "large", vcpu: 8, memoryGb: 16, diskGb: 111 },
    { name: "xlarge", vcpu: 16, memoryGb: 32, diskGb: 251 },
  ],
  second: [
    { name: "standard-2", vcpu: 2, memoryGb: 8, diskGb: 100 },
    { name: "standard-4", vcpu: 4, memoryGb: 16, diskGb: 100 },
    { name: "standard-8", vcpu: 8, memoryGb: 32, diskGb: 150 },
  ],
};

/** The size of a provider with this name, such as the size that ran a job. */
export function providerSizeNamed(
  provider: ProviderKey,
  name: string,
): ProviderSize {
  const size = providerSizes[provider].find((entry) => entry.name === name);
  if (!size) throw new Error(`No ${provider} size ${name}`);
  return size;
}

/** The largest size of a provider, by vCPU. */
export function largestSizeOf(provider: ProviderKey): ProviderSize {
  return providerSizes[provider].reduce((largest, size) =>
    size.vcpu > largest.vcpu ? size : largest,
  );
}

/**
 * The largest size of a provider runs the jobs of a label that nothing fits,
 * with less free disk than the label when need be, but only when its disk
 * holds the image, as the answer to item 77 says.
 */
export function largestHoldsImage(provider: ProviderKey): boolean {
  return holdsImage(largestSizeOf(provider).diskGb);
}

/**
 * The cheapest size of a provider with at least the vCPU and memory of a
 * size and a disk that holds its VM disk, which a sandbox for a job of that
 * size gets. It can be larger than the size.
 */
export function providerSizeFor(
  provider: ProviderKey,
  size: Size,
): ProviderSize | undefined {
  return providerSizes[provider].find(
    (option) =>
      option.vcpu >= size.vcpu &&
      option.memoryGb >= size.memoryGb &&
      option.diskGb * gigabyte >= vmDiskBytes(size),
  );
}

/** A size fits a provider when one of its sizes has the vCPU, memory, and VM disk. */
export function fitsProvider(provider: ProviderKey, size: Size): boolean {
  return providerSizeFor(provider, size) !== undefined;
}
