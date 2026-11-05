// The slots and the job disk of an own machine, by "Slots" and "Disk" in
// docs/protocol/job-routing.md: a slot is 2 vCPU and 8 GB, and the host
// reserve keeps a quarter of the hardware threads and a quarter of the memory.
// The disk first keeps the image budget, and the host reserve keeps a quarter
// of the rest. The cost claim counts the slots and the job disk of one new
// Hetzner AX42.

/** One slot of a machine: the vCPU and memory of the smallest job VM. */
export const slotSize = { vcpu: 2, memoryGb: 8 } as const;

/** One decimal GB, in bytes. */
const gigabyte = 1_000_000_000;

/**
 * The VM disk that each job of the default label ciproxy holds, in bytes: the
 * current image with the 20 GB of free disk of the label. RootSizing in
 * ciproxy-agent gives this value, so change it with the image or that rule.
 */
const defaultJobDiskBytes = 81_635_856_384;

/** Two full images, a download, and image support have a separate budget. */
const imageBudgetGb = 140;

/**
 * Jobs get three quarters of the threads, the memory, and the disk after the
 * image budget; the host keeps the rest.
 */
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
  const afterImages = Math.max(0, hardware.diskGb - imageBudgetGb) * gigabyte;
  const jobDiskBytes = Math.floor(afterImages * jobShare);
  return Math.min(
    machineSlots(hardware),
    Math.floor(jobDiskBytes / defaultJobDiskBytes),
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
