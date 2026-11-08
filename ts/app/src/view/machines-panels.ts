// The panels of the machine page beside its slots: the health checks with
// the values of the last contact marked as old, the machine cache with Clear
// cache, and the monthly cost that the savings report uses. Every value comes
// from the MachineView of GET machines; a value that the machine has not
// reported shows as not reported.

import type { MachineView } from "../api/schema.ts";
import { cacheCopy, costCopy, healthCopy } from "./machines-copy.ts";
import {
  centsToInput,
  formatAgo,
  formatMonthly,
  gigabytes,
} from "./machines-format.ts";
import { cacheHitsText, contactSince } from "./machines.ts";

export type HealthTone = "ok" | "err" | "idle";

export type HealthRow = {
  key: string;
  label: string;
  value: string;
  tone: HealthTone;
  /** The value of the last contact, shown in grey. */
  old: boolean;
};

export type HealthModel = { rows: HealthRow[]; note: string | null };

export function healthModel(machine: MachineView, now: number): HealthModel {
  const old = machine.health_old;
  const contact = machine.last_contact_at ?? null;
  const fresh = (value: string, key: string, label: string): HealthRow => ({
    key,
    label,
    value,
    tone: old ? "idle" : "ok",
    old,
  });
  const disks = machine.disks.map((disk, index) =>
    fresh(
      healthCopy.diskFree(gigabytes(disk.free_bytes)),
      `disk-${index}`,
      machine.disks.length === 1
        ? healthCopy.disk
        : healthCopy.diskAt(disk.path),
    ),
  );
  const rows: HealthRow[] = [
    {
      key: "contact",
      label: healthCopy.lastContact,
      value: contact === null ? healthCopy.noContact : formatAgo(contact, now),
      tone: machine.offline ? "err" : contact === null ? "idle" : "ok",
      old: false,
    },
    machine.kvm
      ? fresh(healthCopy.kvmReady, "kvm", healthCopy.kvm)
      : {
          key: "kvm",
          label: healthCopy.kvm,
          value: healthCopy.kvmMissing,
          tone: "err",
          old: false,
        },
    ...(disks.length > 0
      ? disks
      : [
          {
            ...fresh(healthCopy.noReport, "disk", healthCopy.disk),
            tone: "idle" as const,
          },
        ]),
    machine.agent_version === ""
      ? {
          ...fresh(healthCopy.noReport, "agent", healthCopy.agent),
          tone: "idle",
        }
      : fresh(machine.agent_version, "agent", healthCopy.agent),
  ];
  return {
    rows,
    note: old ? healthCopy.oldValues(contactSince(machine, now)) : null,
  };
}

export type CacheModel =
  | { reported: false; text: string }
  | {
      reported: true;
      ratio: number;
      value: string;
      used: string;
      hits: string;
      docker: string;
    };

export function cacheModel(machine: MachineView): CacheModel {
  const cache = machine.cache ?? null;
  if (cache === null) return { reported: false, text: cacheCopy.unreported };
  const capacity = Number(cache.capacity_bytes);
  return {
    reported: true,
    ratio: capacity > 0 ? Math.min(Number(cache.used_bytes) / capacity, 1) : 0,
    value: cacheCopy.used(
      gigabytes(cache.used_bytes),
      gigabytes(cache.capacity_bytes),
    ),
    used: gigabytes(cache.used_bytes),
    hits: cacheHitsText([machine]),
    docker: cacheCopy.dockerSize(gigabytes(cache.docker_disk_bytes)),
  };
}

/** Clear cache shows for an Admin while the machine is online and reported. */
export function canClearCache(machine: MachineView, admin: boolean): boolean {
  return admin && !machine.offline && (machine.cache ?? null) !== null;
}

export type CostModel = { value: string; caption: string; input: string };

export function costModel(machine: MachineView): CostModel {
  const cents = machine.monthly_cost_cents ?? null;
  return cents === null
    ? { value: costCopy.none, caption: costCopy.needs, input: "" }
    : {
        value: formatMonthly(cents),
        caption: costCopy.uses,
        input: centsToInput(cents),
      };
}
