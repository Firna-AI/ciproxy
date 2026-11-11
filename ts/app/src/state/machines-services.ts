// The Machines stores of one document, beside the shared services: one
// machine endpoints object on the shared client, one machines and one
// overflow store for each workspace, and one change store for each machine.
// They live as long as the services of the document, so a full-document
// navigation drops them, as it drops every other store.

import {
  createMachineEndpoints,
  type MachineEndpoints,
} from "../api/machine-endpoints.ts";
import type { AppServices } from "../services/services.ts";
import {
  createMachineChangeStore,
  type MachineChangeStore,
} from "./machines-changes.ts";
import {
  createOverflowStore,
  type OverflowStore,
} from "./machines-overflow.ts";
import { createMachinesStore, type MachinesStore } from "./machines.ts";

export type MachinesServices = {
  readonly endpoints: MachineEndpoints;
  machines(workspaceId: string): MachinesStore;
  overflow(workspaceId: string): OverflowStore;
  changes(workspaceId: string, machineId: string): MachineChangeStore;
};

function cached<T>(create: (key: string) => T): (key: string) => T {
  const values = new Map<string, T>();
  return (key) => {
    const known = values.get(key);
    if (known !== undefined) return known;
    const value = create(key);
    values.set(key, value);
    return value;
  };
}

function createMachinesServices(app: AppServices): MachinesServices {
  const endpoints = createMachineEndpoints(app.client);
  const report = app.session.report;
  const machines = cached((workspaceId) =>
    createMachinesStore({
      endpoints,
      workspaceId,
      report,
      timers: app.env.timers,
      visibility: app.env.visibility,
    }),
  );
  const overflow = cached((workspaceId) =>
    createOverflowStore({ endpoints, workspaceId, report }),
  );
  const changes = cached((workspaceId) =>
    cached((machineId) =>
      createMachineChangeStore({
        endpoints,
        workspaceId,
        machineId,
        machines: machines(workspaceId),
        mutate: app.session.mutate,
      }),
    ),
  );
  return {
    endpoints,
    machines,
    overflow,
    changes: (workspaceId, machineId) => changes(workspaceId)(machineId),
  };
}

const registry = new WeakMap<AppServices, MachinesServices>();

/** The Machines stores of the document that `app` serves. */
export function machinesServices(app: AppServices): MachinesServices {
  const known = registry.get(app);
  if (known !== undefined) return known;
  const created = createMachinesServices(app);
  registry.set(app, created);
  return created;
}
