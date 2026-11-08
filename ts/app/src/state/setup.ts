// Setup of one workspace: GET setup and GET machines, read together, and
// read again while Setup waits for a machine or for the first job. A failed
// refresh keeps the last good view and shows the failure. Each workspace
// has its own store, so a page for one workspace never shows another one.

import type { ApiFailure, ApiResult } from "../api/client.ts";
import type { Endpoints } from "../api/endpoints.ts";
import type { MachineView, SetupView } from "../api/schema.ts";
import { createPoller, type Timers, type Visibility } from "./poller.ts";
import { createStore, type Store } from "./store.ts";

export type SetupSnapshot =
  | { status: "loading" }
  | {
      status: "ready";
      setup: SetupView;
      machines: MachineView[];
      refreshFailure: ApiFailure | null;
    }
  | { status: "failed"; failure: ApiFailure };

export type SetupStore = {
  readonly state: Store<SetupSnapshot>;
  /** Read Setup now. Returns whether the page still waits for something. */
  load(): Promise<boolean>;
  /** Read Setup once for this document; later calls share that read. */
  ensure(): Promise<boolean>;
  /** Read on the interval while Setup waits. Returns the stop function. */
  watch(): () => void;
};

/** Setup waits while no machine has joined or the first job has not ended. */
export function setupWaits(setup: SetupView): boolean {
  return (
    setup.waiting_for_machine ||
    setup.machines === 0 ||
    setup.first_job?.phase !== "completed"
  );
}

export const setupPollInterval = 5_000;

export function createSetupStore(options: {
  endpoints: Endpoints;
  workspaceId: string;
  report: <T>(result: ApiResult<T>) => ApiResult<T>;
  timers: Timers;
  visibility: Visibility;
}): SetupStore {
  const { endpoints, workspaceId, report } = options;
  const state = createStore<SetupSnapshot>({ status: "loading" });
  let first: Promise<boolean> | null = null;

  const fail = (failure: ApiFailure): boolean => {
    const current = state.get();
    state.set(
      current.status === "ready"
        ? { ...current, refreshFailure: failure }
        : { status: "failed", failure },
    );
    return true;
  };

  const load = async (): Promise<boolean> => {
    const [setup, machines] = await Promise.all([
      endpoints.setup(workspaceId).then(report),
      endpoints.machines(workspaceId).then(report),
    ]);
    if (!setup.ok) return fail(setup.failure);
    if (!machines.ok) return fail(machines.failure);
    state.set({
      status: "ready",
      setup: setup.value,
      machines: machines.value,
      refreshFailure: null,
    });
    return setupWaits(setup.value);
  };

  const poller = createPoller(
    load,
    setupPollInterval,
    options.timers,
    options.visibility,
  );

  return {
    state,
    load,
    ensure() {
      first ??= load();
      return first;
    },
    watch() {
      poller.start();
      return () => poller.stop();
    },
  };
}
