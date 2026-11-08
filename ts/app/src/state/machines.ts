// The machines of one workspace, as GET machines gives them. The list, the
// machine page, and Add a machine share this store, so a page change between
// them shows the last read at once. A page that shows the machines watches
// the store, which then reads again on an interval. A failed refresh keeps
// the last good read and shows the failure. Only the newest read counts, so
// an old answer never undoes a change that a later read already shows, and a
// machine that the person removed leaves the list at once.

import type { ApiFailure, ApiResult } from "../api/client.ts";
import type { MachineEndpoints } from "../api/machine-endpoints.ts";
import type { MachineView } from "../api/schema.ts";
import { createPoller, type Timers, type Visibility } from "./poller.ts";
import { createStore, type Store } from "./store.ts";

export type MachinesSnapshot =
  | { status: "loading" }
  | {
      status: "ready";
      machines: MachineView[];
      refreshFailure: ApiFailure | null;
    }
  | { status: "failed"; failure: ApiFailure };

export type MachinesStore = {
  readonly state: Store<MachinesSnapshot>;
  /** Read the machines now. A later read makes this one count for nothing. */
  load(): Promise<void>;
  /** Read every `interval` ms while a page shows them. Returns the stop. */
  watch(interval: number): () => void;
  /** Take a removed machine out of the list before the next read. */
  forget(machineId: string): void;
};

/** The list and the machine page read again at the report pace of agents. */
export const machinesPollInterval = 10_000;

export function createMachinesStore(options: {
  endpoints: MachineEndpoints;
  workspaceId: string;
  report: <T>(result: ApiResult<T>) => ApiResult<T>;
  timers: Timers;
  visibility: Visibility;
}): MachinesStore {
  const { endpoints, workspaceId, report } = options;
  const state = createStore<MachinesSnapshot>({ status: "loading" });
  let latest = 0;

  const load = async (): Promise<void> => {
    latest += 1;
    const mine = latest;
    const result = report(await endpoints.machines(workspaceId));
    if (mine !== latest) return;
    if (result.ok) {
      state.set({
        status: "ready",
        machines: result.value,
        refreshFailure: null,
      });
      return;
    }
    const current = state.get();
    state.set(
      current.status === "ready"
        ? { ...current, refreshFailure: result.failure }
        : { status: "failed", failure: result.failure },
    );
  };

  return {
    state,
    load,
    watch(interval) {
      const poller = createPoller(
        async () => {
          await load();
          return true;
        },
        interval,
        options.timers,
        options.visibility,
      );
      poller.start();
      return () => poller.stop();
    },
    forget(machineId) {
      latest += 1;
      const current = state.get();
      if (current.status !== "ready") return;
      state.set({
        ...current,
        machines: current.machines.filter((item) => item.id !== machineId),
      });
    },
  };
}
