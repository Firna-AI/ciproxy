// The savings report of one workspace for one UTC range. While the API says
// that its refresh raced another one, the store reads again on an interval
// until the report is complete. A failed read keeps the shown report of the
// same range with the failure. Only the newest read can change the view, so
// a late answer of an older range never replaces a newer one.

import type { ApiFailure, ApiResult } from "../api/client.ts";
import type { SavingsPage, SavingsRange } from "../api/savings-types.ts";
import { createPoller, type Timers, type Visibility } from "./poller.ts";
import { createStore, type Store } from "./store.ts";

export type SavingsSnapshot =
  | { status: "loading"; range: SavingsRange }
  | {
      status: "ready";
      range: SavingsRange;
      page: SavingsPage;
      refreshFailure: ApiFailure | null;
    }
  | { status: "failed"; range: SavingsRange; failure: ApiFailure };

export type SavingsStore = {
  readonly state: Store<SavingsSnapshot>;
  /** Show this range: read it now, and again until it is complete. */
  select(range: SavingsRange): Promise<void>;
  /** Stop reading again, such as when the page closes. */
  stop(): void;
};

export const savingsRefreshInterval = 2_000;

export function createSavingsStore(options: {
  read: (range: SavingsRange) => Promise<ApiResult<SavingsPage>>;
  report: <T>(result: ApiResult<T>) => ApiResult<T>;
  timers: Timers;
  visibility: Visibility;
}): SavingsStore {
  const { read, report } = options;
  let range: SavingsRange = "last30_days";
  const state = createStore<SavingsSnapshot>({ status: "loading", range });
  let latest = 0;

  /** Read the current range. A late answer of an older read changes nothing. */
  const load = async (): Promise<void> => {
    latest += 1;
    const mine = latest;
    const asked = range;
    const result = report(await read(asked));
    if (mine !== latest) return;
    if (result.ok) {
      state.set({
        status: "ready",
        range: asked,
        page: result.value,
        refreshFailure: null,
      });
      return;
    }
    const current = state.get();
    state.set(
      current.status === "ready" && current.range === asked
        ? { ...current, refreshFailure: result.failure }
        : { status: "failed", range: asked, failure: result.failure },
    );
  };

  /**
   * Whether the shown report still waits for a complete refresh. It reads the
   * current state, not the answer of one read, so a late answer of an older
   * range can never stop the refresh of the range that shows now.
   */
  const refreshing = (): boolean => {
    const current = state.get();
    return current.status === "ready" && !current.page.refresh_complete;
  };

  const poller = createPoller(
    async () => {
      await load();
      return refreshing();
    },
    savingsRefreshInterval,
    options.timers,
    options.visibility,
  );

  return {
    state,
    async select(next) {
      // End the previous refresh cycle first. The poller keeps its tab
      // watcher after a cycle ends, and a new start would replace it.
      poller.stop();
      range = next;
      const current = state.get();
      if (current.range !== next || current.status === "failed") {
        state.set({ status: "loading", range: next });
      }
      await load();
      if (refreshing()) poller.start();
    },
    stop() {
      poller.stop();
    },
  };
}
