// The what-ifs of Machines with figures that a machine did not measure, as the
// API gives them: a machine cache or Docker disks with the status unavailable
// and no figures. While its Docker disks are unavailable, the API knows no job
// disk of the machine, so it gives no jobs at a time and no disk limit.

import type { components } from "../../api/index.ts";
import { machineIds } from "./m20-fixtures-machines.ts";
import type { M20Scenario } from "./m20-harness-state.ts";

type Machine = components["schemas"]["MachineView"];

const unavailable = { status: "unavailable" } as const;

/** The machine, when it did not measure its machine cache for its report. */
function withoutMachineCache(machine: Machine): Machine {
  const cache = machine.cache ?? null;
  if (cache === null) return machine;
  return { ...machine, cache: { ...cache, machine_cache: unavailable } };
}

/** The machine, when its agent could not read its Docker disks. */
function withoutDockerDisks(machine: Machine): Machine {
  const cache = machine.cache ?? null;
  if (cache === null) return machine;
  return {
    ...machine,
    cache: { ...cache, docker_disks: unavailable },
    capacity_known: false,
    concurrent_default_jobs: null,
    disk_limited: null,
    job_disk_bytes: null,
  };
}

/** forge-01 changed by `change`, and the other machines as they are. */
function primary(
  machines: Machine[],
  change: (machine: Machine) => Machine,
): Machine[] {
  return machines.map((machine) =>
    machine.id === machineIds.primary ? change(machine) : machine,
  );
}

/**
 * The machines of a scenario. In machine-cache-unknown, the list keeps
 * forge-01, office-mini-1 with its measured cache, and lab-no-kvm with no
 * report, so the cache hits cover office-mini-1 alone and name the others.
 */
export function statisticsMachines(
  scenario: M20Scenario,
  machines: Machine[],
): Machine[] {
  switch (scenario) {
    case "machine-cache-unknown":
      return primary(machines, withoutMachineCache).filter((machine) =>
        [machineIds.primary, machineIds.noKvm, machineIds.diskLimited].some(
          (id) => id === machine.id,
        ),
      );
    case "machine-docker-unknown":
      return primary(machines, withoutDockerDisks);
    case "machine-figures-unknown":
    case "viewer-figures-unknown":
    case "billing-figures-unknown":
      return primary(machines, (machine) =>
        withoutDockerDisks(withoutMachineCache(machine)),
      );
    case "machines-no-cache-measured":
      return machines.map(withoutMachineCache);
    default:
      return machines;
  }
}
