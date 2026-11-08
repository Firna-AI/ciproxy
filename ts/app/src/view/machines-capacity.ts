// The capacity of a machine for jobs of the label ciproxy, as the API
// computes it: the jobs that run at a time by its slots and its disk, the
// limit of a disk that holds fewer of them than the slots, and the lines of a
// machine that just joined. The page words the numbers of the API and
// computes none of them.

import type { MachineView } from "../api/schema.ts";
import { addCopy, slotCopy } from "./machines-copy.ts";
import { dotJoin, gigabytes, unbroken } from "./machines-format.ts";
import { machineStatus } from "./machines.ts";

/**
 * The jobs of the label ciproxy that the machine runs at a time, by its slots
 * and its disk, as the API computes them; null while the capacity is not
 * known.
 */
export function concurrentJobs(machine: MachineView): number | null {
  const status = machineStatus(machine);
  if (status === "offline" || status === "no-slots") return null;
  const jobs = machine.concurrent_default_jobs ?? null;
  if (!machine.capacity_known || jobs === null) return null;
  return Number(jobs);
}

export type DiskLimit = {
  /** The jobs of ciproxy that the disk holds at a time. */
  jobs: number;
  /** The whole GB of the VM disk of one such job, or null when unknown. */
  vmDisk: string | null;
  /** The whole GB of the disk for jobs, or null when unknown. */
  jobDisk: string | null;
};

/**
 * The limit of a machine whose disk holds fewer jobs of ciproxy than its
 * slots, as the API reports it; null when the API does not say so.
 */
export function diskLimit(machine: MachineView): DiskLimit | null {
  const jobs = machine.concurrent_default_jobs ?? null;
  if (machine.disk_limited !== true || jobs === null) return null;
  const vmDisk = machine.default_root_disk_bytes ?? null;
  const jobDisk = machine.job_disk_bytes ?? null;
  return {
    jobs: Number(jobs),
    vmDisk: vmDisk === null ? null : gigabytes(vmDisk),
    jobDisk: jobDisk === null ? null : gigabytes(jobDisk),
  };
}

/**
 * The line under a machine that just joined: the jobs of ciproxy that its
 * disk holds when the disk sets that number, else the jobs that fit, or null
 * while the API does not know them.
 */
export function joinedFits(machine: MachineView): string | null {
  const limit = diskLimit(machine);
  if (limit !== null) return addCopy.diskHolds(limit.jobs);
  const jobs = concurrentJobs(machine);
  return jobs === null ? null : slotCopy.fits(jobs);
}

/** "16 threads · 64 GB · KVM ready · 6 slots": what a new machine reported. */
export function connectedFacts(machine: MachineView): string {
  return dotJoin(
    unbroken(`${machine.threads} threads`),
    unbroken(`${gigabytes(machine.memory_bytes)} GB`),
    unbroken("KVM ready"),
    slotCopy.unreported(machine.slots),
  );
}
