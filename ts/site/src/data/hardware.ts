// The slots of an own machine, by "Slots" in docs/protocol/job-routing.md: a
// slot is 2 vCPU and 8 GB, and the host reserve keeps a quarter of the
// hardware threads and a quarter of the memory. The cost claim counts the
// slots of one new Hetzner AX42.

/** One slot of a machine: the vCPU and memory of the smallest job VM. */
export const slotSize = { vcpu: 2, memoryGb: 8 } as const;

/** The default job's full disk reservation, apart from the image budget. */
export const defaultJobDiskGb = 80;

/** Two full images, a download, and image support have a separate budget. */
const imageBudgetGb = 140;

/** Jobs get three quarters of CPU and memory; the host keeps the rest. */
const jobShare = 3 / 4;

/** The hardware of a machine that the slot rule reads. */
export type Hardware = {
  cpu: string;
  threads: number;
  memoryGb: number;
  diskGb: number;
};

/** The slots of a machine: the smaller of its thread and memory slots. */
export function machineSlots(
  hardware: Pick<Hardware, "threads" | "memoryGb">,
): number {
  return Math.min(
    Math.floor((hardware.threads * jobShare) / slotSize.vcpu),
    Math.floor((hardware.memoryGb * jobShare) / slotSize.memoryGb),
  );
}

/** Concurrent default jobs after the image budget and host disk reserve. */
export function machineConcurrentJobs(
  hardware: Pick<Hardware, "threads" | "memoryGb" | "diskGb">,
): number {
  const jobDiskGb = Math.max(0, hardware.diskGb - imageBudgetGb) * jobShare;
  return Math.min(
    machineSlots(hardware),
    Math.floor(jobDiskGb / defaultJobDiskGb),
  );
}

/** A new Hetzner AX42, as the Hetzner page lists it. */
export const ax42: Hardware = {
  cpu: "AMD Ryzen 7 PRO 8700GE",
  threads: 16,
  memoryGb: 64,
  /** Both 512 GB disks form one filesystem for this calculation. */
  diskGb: 1_024,
};
