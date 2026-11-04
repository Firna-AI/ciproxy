// The slots of an own machine, by "Slots" in docs/protocol/job-routing.md: a
// slot is 2 vCPU and 8 GB, and the host reserve keeps a quarter of the
// hardware threads and a quarter of the memory. The cost claim counts the
// slots of one new Hetzner AX42.

/** One slot of a machine: the vCPU and memory of the smallest job VM. */
export const slotSize = { vcpu: 2, memoryGb: 8 } as const;

/** Jobs get three quarters of a machine; the host reserve keeps the rest. */
const jobShare = 3 / 4;

/** The hardware of a machine that the slot rule reads. */
export type Hardware = { cpu: string; threads: number; memoryGb: number };

/** The slots of a machine: the smaller of its thread and memory slots. */
export function machineSlots(
  hardware: Pick<Hardware, "threads" | "memoryGb">,
): number {
  return Math.min(
    Math.floor((hardware.threads * jobShare) / slotSize.vcpu),
    Math.floor((hardware.memoryGb * jobShare) / slotSize.memoryGb),
  );
}

/** A new Hetzner AX42, as the Hetzner page lists it. */
export const ax42: Hardware = {
  cpu: "AMD Ryzen 7 PRO 8700GE",
  threads: 16,
  memoryGb: 64,
};
