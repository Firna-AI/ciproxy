// The overflow summary of one workspace, which every role reads for the
// Overflow panel of Machines and for where jobs go while a machine takes
// none. A failed read shows with Try again; it never hides the machines.

import type { ApiFailure, ApiResult } from "../api/client.ts";
import type { MachineEndpoints } from "../api/machine-endpoints.ts";
import type { OverflowSettings } from "../api/machine-schema.ts";
import { createStore, type Store } from "./store.ts";

export type OverflowSnapshot =
  | { status: "loading" }
  | { status: "ready"; summary: OverflowSettings }
  | { status: "failed"; failure: ApiFailure };

export type OverflowStore = {
  readonly state: Store<OverflowSnapshot>;
  /** Read the summary now. A later read makes this one count for nothing. */
  load(): Promise<void>;
  /** Read once for this document; later calls share that read. */
  ensure(): Promise<void>;
};

/** The summary that a snapshot holds, or null while none is known. */
export function overflowSummary(
  snapshot: OverflowSnapshot,
): OverflowSettings | null {
  return snapshot.status === "ready" ? snapshot.summary : null;
}

export function createOverflowStore(options: {
  endpoints: MachineEndpoints;
  workspaceId: string;
  report: <T>(result: ApiResult<T>) => ApiResult<T>;
}): OverflowStore {
  const { endpoints, workspaceId, report } = options;
  const state = createStore<OverflowSnapshot>({ status: "loading" });
  let latest = 0;
  let first: Promise<void> | null = null;

  const load = async (): Promise<void> => {
    latest += 1;
    const mine = latest;
    const result = report(await endpoints.overflowSummary(workspaceId));
    if (mine !== latest) return;
    if (result.ok) {
      state.set({ status: "ready", summary: result.value });
    } else if (state.get().status !== "ready") {
      state.set({ status: "failed", failure: result.failure });
    }
  };

  return {
    state,
    load,
    ensure() {
      first ??= load();
      return first;
    },
  };
}
