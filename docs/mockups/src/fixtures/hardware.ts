// The hardware of the own machines of mokly-ai, and the rule of design call
// 34 as the plan defines it. A slot is 2 vCPU and 8 GB. The host reserve
// keeps a quarter of the threads and the memory. The disk first keeps the
// image budget for the VM images, as the answer to item 73 says, and the
// host reserve keeps a quarter of the rest. A size fits a machine when the
// machine has enough free slots and enough free disk for it: each running
// job holds the disk of its size, and the machine cache uses the rest.

import { dotJoin, unbroken } from "./format.ts";
import type { Size } from "./sizes.ts";

/** One slot of a machine: the vCPU and memory of the smallest job VM. */
export const slotSize = { vcpu: 2, memoryGb: 8 } as const;

/** Jobs get three quarters of a machine; the host reserve keeps the rest. */
const jobShare = 3 / 4;

/** The disk for two VM images and one download, apart from the job disk. */
export const imageBudgetGb = 140;

/** What the agent reports about a machine, and the cost that mokly-ai set. */
export type Hardware = {
  cpu: string;
  threads: number;
  memoryGb: number;
  /** The disks, as the machine page names them. */
  disk: string;
  /** The space of the disks, in GB. */
  diskGb: number;
  /** The monthly cost that the savings report uses. */
  monthlyCost: number;
};

/** The slots of a machine, by the rule of Slots in job-routing.md. */
export function machineSlots(hardware: Hardware): number {
  return Math.min(
    Math.floor((hardware.threads * jobShare) / slotSize.vcpu),
    Math.floor((hardware.memoryGb * jobShare) / slotSize.memoryGb),
  );
}

/** The disk that running jobs can hold, after the images and host reserve. */
export function jobDiskGb(hardware: Hardware): number {
  const afterImages = Math.max(0, hardware.diskGb - imageBudgetGb);
  return Math.floor(afterImages * jobShare);
}

/** The slots that a job VM of this size takes on a machine. */
export function slotsFor(size: Pick<Size, "vcpu" | "memoryGb">): number {
  return Math.max(
    Math.ceil(size.vcpu / slotSize.vcpu),
    Math.ceil(size.memoryGb / slotSize.memoryGb),
  );
}

/** How many jobs of a size fit on a machine at a time: by slots and disk. */
export function jobsAtOnce(
  hardware: Hardware,
  slots: number,
  size: Size,
): number {
  return Math.min(
    Math.floor(slots / slotsFor(size)),
    Math.floor(jobDiskGb(hardware) / size.diskGb),
  );
}

/**
 * A Hetzner AX42, the same for each of the three machines of mokly-ai. The
 * monthly cost is what mokly-ai entered for its machines, which it bought at
 * auction.
 */
export const ax42: Hardware = {
  cpu: "AMD Ryzen 7 PRO 8700GE",
  threads: 16,
  memoryGb: 64,
  disk: unbroken("2 × 512 GB NVMe"),
  diskGb: 1024,
  monthlyCost: 53.4,
};

/** office-mini-1, a mini PC in the office of mokly-ai, with one small disk. */
export const miniPc: Hardware = {
  cpu: "AMD Ryzen 9 7940HS",
  threads: 16,
  memoryGb: 64,
  disk: unbroken("512 GB NVMe"),
  diskGb: 512,
  monthlyCost: 25,
};

/** "16 threads · 64 GB", the line under a machine name. */
export function hardwareSize(hardware: Hardware): string {
  return dotJoin(
    unbroken(`${hardware.threads} threads`),
    unbroken(`${hardware.memoryGb} GB`),
  );
}
