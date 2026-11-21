// The changes that an Admin makes to one machine: a drain and its end, a
// removal, Clear cache, the warm runner target, and the monthly cost. One
// change runs at a time, and a second press while one runs does nothing. A
// change that succeeds reads the machines again before it reports done, so
// the page never shows the old state beside the new outcome. A removed
// machine leaves the list at once. A failure stays visible until the next
// change or a dismissal.

import type { ApiFailure, ApiResult } from "../api/client.ts";
import type { MachineEndpoints } from "../api/machine-endpoints.ts";
import type { MachineCommand } from "../api/machine-schema.ts";
import type { MachinesStore } from "./machines.ts";
import { createStore, type Store } from "./store.ts";

export type MachineChange =
  | { kind: "command"; command: MachineCommand }
  | { kind: "warm-runners"; count: number }
  | { kind: "cost"; cents: string };

export type ChangeSnapshot =
  | { status: "idle" }
  | { status: "busy"; change: MachineChange }
  | {
      status: "done";
      change: MachineChange;
      /** For a command: whether the connected agent took it at once. */
      connected: boolean | null;
    }
  | { status: "failed"; change: MachineChange; failure: ApiFailure };

export type MachineChangeStore = {
  readonly state: Store<ChangeSnapshot>;
  /** Run a change unless one runs. Returns whether it succeeded. */
  run(change: MachineChange): Promise<boolean>;
  /** Forget the outcome of the last change. */
  dismiss(): void;
};

type Outcome = ApiResult<{ connected: boolean | null }>;

export function createMachineChangeStore(options: {
  endpoints: MachineEndpoints;
  workspaceId: string;
  machineId: string;
  machines: MachinesStore;
  mutate: <T>(run: () => Promise<ApiResult<T>>) => Promise<ApiResult<T>>;
}): MachineChangeStore {
  const { endpoints, workspaceId, machineId, machines, mutate } = options;
  const state = createStore<ChangeSnapshot>({ status: "idle" });

  const send = async (change: MachineChange): Promise<Outcome> => {
    switch (change.kind) {
      case "command": {
        const result = await mutate(() =>
          endpoints.command(workspaceId, machineId, change.command),
        );
        return result.ok
          ? { ok: true, value: { connected: result.value.connected } }
          : result;
      }
      case "warm-runners": {
        const result = await mutate(() =>
          endpoints.setWarmRunners(workspaceId, machineId, change.count),
        );
        return result.ok ? { ok: true, value: { connected: null } } : result;
      }
      case "cost": {
        const result = await mutate(() =>
          endpoints.setMonthlyCost(workspaceId, machineId, change.cents),
        );
        return result.ok ? { ok: true, value: { connected: null } } : result;
      }
    }
  };

  return {
    state,
    async run(change) {
      if (state.get().status === "busy") return false;
      state.set({ status: "busy", change });
      const result = await send(change);
      if (!result.ok) {
        state.set({ status: "failed", change, failure: result.failure });
        return false;
      }
      if (change.kind === "command" && change.command === "remove") {
        machines.forget(machineId);
      } else {
        await machines.load();
      }
      state.set({ status: "done", change, connected: result.value.connected });
      return true;
    },
    dismiss() {
      if (state.get().status !== "busy") state.set({ status: "idle" });
    },
  };
}
