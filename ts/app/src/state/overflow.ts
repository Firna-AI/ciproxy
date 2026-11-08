// The Boat settings of one workspace, for Setup and Settings under Overflow.
// The store reads them on request and after each accepted change. A change
// that the API accepted is never sent again: when the read after it fails,
// the store shows the failed read with the accepted change, and Try again
// only reads. The key of a connection passes through and is never kept.

import type { ApiFailure, ApiResult } from "../api/client.ts";
import type {
  BoatKeyInput,
  OverflowEndpoints,
} from "../api/overflow-endpoints.ts";
import type {
  OverflowPolicy,
  OverflowSettings,
  Success,
} from "../api/schema.ts";
import { createStore, type Store } from "./store.ts";

/** A change that the API accepted. */
export type AcceptedChange = "connected" | "disconnected" | "saved";

export type OverflowSnapshot =
  | { status: "loading" }
  | {
      status: "ready";
      settings: OverflowSettings;
      refreshFailure: ApiFailure | null;
    }
  | { status: "failed"; failure: ApiFailure; accepted: AcceptedChange | null };

export type OverflowStore = {
  readonly state: Store<OverflowSnapshot>;
  /** Read the settings now. A read on its way answers this call too. */
  load(): Promise<void>;
  /** Read once for this document; later calls share that read. */
  ensure(): Promise<void>;
  /**
   * Connect a key with a limit. A refused change saves nothing.
   * `onAccepted` runs once the API accepted the key, before the new read, so
   * the form can drop the key at once.
   */
  connect(
    input: BoatKeyInput,
    onAccepted?: () => void,
  ): Promise<ApiResult<Success>>;
  disconnect(): Promise<ApiResult<Success>>;
  /** Save the complete policy: the switch, the wait, and the limit. */
  save(policy: OverflowPolicy): Promise<ApiResult<Success>>;
};

export function createOverflowStore(options: {
  endpoints: OverflowEndpoints;
  workspaceId: string;
  report: <T>(result: ApiResult<T>) => ApiResult<T>;
  mutate: <T>(run: () => Promise<ApiResult<T>>) => Promise<ApiResult<T>>;
}): OverflowStore {
  const { endpoints, workspaceId, report, mutate } = options;
  const state = createStore<OverflowSnapshot>({ status: "loading" });
  let generation = 0;
  let pending: Promise<void> | null = null;
  let first: Promise<void> | null = null;

  const read = async (accepted: AcceptedChange | null): Promise<void> => {
    generation += 1;
    const mine = generation;
    const result = report(await endpoints.overflow(workspaceId));
    if (mine !== generation) return;
    if (result.ok) {
      state.set({
        status: "ready",
        settings: result.value,
        refreshFailure: null,
      });
      return;
    }
    const current = state.get();
    if (accepted === null && current.status === "ready") {
      state.set({ ...current, refreshFailure: result.failure });
      return;
    }
    const kept = current.status === "failed" ? current.accepted : null;
    state.set({
      status: "failed",
      failure: result.failure,
      accepted: accepted ?? kept,
    });
  };

  const load = (): Promise<void> => {
    if (pending !== null) return pending;
    const request = read(null).finally(() => {
      if (pending === request) pending = null;
    });
    pending = request;
    return request;
  };

  const change = async (
    accepted: AcceptedChange,
    run: () => Promise<ApiResult<Success>>,
    onAccepted?: () => void,
  ): Promise<ApiResult<Success>> => {
    const result = await mutate(run);
    if (!result.ok) return result;
    onAccepted?.();
    await read(accepted);
    return result;
  };

  return {
    state,
    load,
    ensure() {
      first ??= load();
      return first;
    },
    connect: (input, onAccepted) =>
      change(
        "connected",
        () => endpoints.connectBoat(workspaceId, input),
        onAccepted,
      ),
    disconnect: () =>
      change("disconnected", () => endpoints.disconnectBoat(workspaceId)),
    save: (policy) =>
      change("saved", () => endpoints.saveOverflow(workspaceId, policy)),
  };
}
