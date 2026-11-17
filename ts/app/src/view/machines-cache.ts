// The cache figures of the Machines pages, from the cache of GET machines. A
// null cache means that ciproxy holds no report of the machine. Otherwise the
// machine cache and the Docker disks are each measured or unavailable, apart.
// A measured zero is a true zero, and an unavailable part shows Unknown, never
// a zero or a share. Each reader checks the generated status itself.

import type { CacheView, MachineView } from "../api/schema.ts";
import { cacheCopy, listCopy } from "./machines-copy.ts";
import { gigabytes } from "./machines-format.ts";

/** A machine cache that the machine measured for its last report. */
export type MeasuredCache = Extract<
  CacheView["machine_cache"],
  { status: "measured" }
>;

/** Docker disks that the machine measured for its last report. */
export type MeasuredDocker = Extract<
  CacheView["docker_disks"],
  { status: "measured" }
>;

/** "83 of 191 GB": the use and the space of a measured machine cache. */
export function cacheUse(figures: MeasuredCache): string {
  return cacheCopy.used(
    gigabytes(figures.used_bytes),
    gigabytes(figures.capacity_bytes),
  );
}

/** The share of its space that a measured machine cache uses, for a meter. */
export function cacheRatio(figures: MeasuredCache): number {
  const capacity = Number(figures.capacity_bytes);
  return capacity > 0 ? Math.min(Number(figures.used_bytes) / capacity, 1) : 0;
}

/** "3 repositories · 18 GB": the Docker disks that the machine measured. */
export function dockerUse(figures: MeasuredDocker): string {
  return cacheCopy.dockerUse(
    figures.repositories,
    gigabytes(figures.used_bytes),
  );
}

/**
 * The share of the cache restores that the machine cache served, rounded
 * down so it never overstates, or the words for no restores.
 */
function share(hits: bigint, restores: bigint): string {
  if (restores <= BigInt(0)) return listCopy.noRestores;
  return `${(hits * BigInt(100)) / restores}%`;
}

/** The cache hits of the last 7 days of one measured machine cache. */
export function machineHits(figures: MeasuredCache): string {
  return share(
    BigInt(figures.hits_last_7_days),
    BigInt(figures.restores_last_7_days),
  );
}

/** The note under the cache figures that names the parts that show Unknown. */
export function unknownNote(cache: CacheView): string | null {
  const cacheUnknown = cache.machine_cache.status === "unavailable";
  const dockerUnknown = cache.docker_disks.status === "unavailable";
  if (cacheUnknown && dockerUnknown) return cacheCopy.unknownBoth;
  if (cacheUnknown) return cacheCopy.unknownCache;
  if (dockerUnknown) return cacheCopy.unknownDocker;
  return null;
}

/** The cache cell of the machines table: the use, Unknown, or no report. */
export function cacheText(machine: MachineView): string {
  const cache = machine.cache ?? null;
  if (cache === null) return cacheCopy.noReport;
  const figures = cache.machine_cache;
  return figures.status === "measured" ? cacheUse(figures) : cacheCopy.unknown;
}

/** The body of the Clear cache dialog, with the size only when measured. */
export function clearCacheBody(machine: MachineView): string {
  const cache = machine.cache ?? null;
  const figures = cache === null ? null : cache.machine_cache;
  return cacheCopy.dialogBody(
    figures !== null && figures.status === "measured"
      ? gigabytes(figures.used_bytes)
      : null,
  );
}

/**
 * The cache hits of Machines: unknown when no machine measured its machine
 * cache, else the share of the measured machines, with the hostnames of the
 * others in list order.
 */
export type CacheHits =
  { kind: "unknown" } | { kind: "share"; value: string; excluded: string[] };

/**
 * The cache hits of the last 7 days on Machines. The share sums the hits and
 * the restores of the machines that measured their machine cache. It leaves
 * out a machine with no report or with unavailable figures, and names it.
 */
export function cacheHits(
  machines: readonly Pick<MachineView, "cache" | "hostname">[],
): CacheHits {
  let hits = BigInt(0);
  let restores = BigInt(0);
  const excluded: string[] = [];
  for (const machine of machines) {
    const cache = machine.cache ?? null;
    const figures = cache === null ? null : cache.machine_cache;
    if (figures === null || figures.status === "unavailable") {
      excluded.push(machine.hostname);
      continue;
    }
    hits += BigInt(figures.hits_last_7_days);
    restores += BigInt(figures.restores_last_7_days);
  }
  if (excluded.length === machines.length) return { kind: "unknown" };
  return { kind: "share", value: share(hits, restores), excluded };
}
