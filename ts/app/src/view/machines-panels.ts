// The panels of the machine page beside its slots: the health checks with
// the values of the last contact marked as old, who may clear the machine
// cache, and the monthly cost that the savings report uses. Every value comes
// from the MachineView of GET machines; a value that the machine has not
// reported shows as not reported. machines-cache.ts gives the cache figures.

import type { MachineView } from "../api/schema.ts";
import { costCopy, healthCopy } from "./machines-copy.ts";
import {
  centsToInput,
  formatAgo,
  formatMonthly,
  gigabytes,
} from "./machines-format.ts";
import { contactSince } from "./machines.ts";

export type HealthTone = "ok" | "warn" | "err" | "idle";

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
          tone: old ? "idle" : "err",
          old,
        },
    ...(disks.length > 0
      ? disks
      : [
          {
            ...fresh(healthCopy.noReport, "disk", healthCopy.disk),
            tone: "idle" as const,
          },
        ]),
    agentRow(machine, fresh),
  ];
  return {
    rows,
    note: old ? healthCopy.oldValues(contactSince(machine, now)) : null,
  };
}

/**
 * The agent version, and whether it is the newest release, as the API
 * compares them. An old report claims nothing, because the agent can have
 * updated since; a release that ciproxy could not read claims nothing too.
 */
function agentRow(
  machine: MachineView,
  fresh: (value: string, key: string, label: string) => HealthRow,
): HealthRow {
  const version = machine.agent_version;
  if (version === "") {
    return {
      ...fresh(healthCopy.noReport, "agent", healthCopy.agent),
      tone: "idle",
    };
  }
  if (machine.health_old) return fresh(version, "agent", healthCopy.agent);
  const latest = machine.latest_agent_version ?? null;
  const current = machine.agent_up_to_date ?? null;
  if (current === true) {
    return fresh(healthCopy.upToDate(version), "agent", healthCopy.agent);
  }
  if (current === false && latest !== null) {
    return {
      ...fresh(healthCopy.behind(version, latest), "agent", healthCopy.agent),
      tone: "warn",
    };
  }
  return { ...fresh(version, "agent", healthCopy.agent), tone: "idle" };
}

/**
 * Clear cache shows for an Admin while the machine is online and has a stored
 * report, also while its figures are unavailable, because a clear needs none.
 */
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
