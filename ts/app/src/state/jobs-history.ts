// The finished jobs of one workspace for one set of filters. A new filter
// reads again and keeps the shown list until its own answer arrives, so the
// page does not flash. Only the answer of the newest filter can change the
// view, so a late answer of an older search never replaces a newer one.

import type { ApiFailure, ApiResult } from "../api/client.ts";
import type { HistoryQuery, JobsHistoryPage } from "../api/jobs-types.ts";
import { createStore, type Store } from "./store.ts";

export type HistoryResult = { query: HistoryQuery; page: JobsHistoryPage };

export type HistorySnapshot =
  | { status: "loading"; query: HistoryQuery; shown: HistoryResult | null }
  | { status: "ready"; query: HistoryQuery; page: JobsHistoryPage }
  | { status: "failed"; query: HistoryQuery; failure: ApiFailure };

export type HistoryStore = {
  readonly state: Store<HistorySnapshot>;
  /** Read the finished jobs that match these filters. */
  load(query: HistoryQuery): Promise<void>;
};

export function createHistoryStore(options: {
  read: (query: HistoryQuery) => Promise<ApiResult<JobsHistoryPage>>;
  report: <T>(result: ApiResult<T>) => ApiResult<T>;
}): HistoryStore {
  const { read, report } = options;
  const state = createStore<HistorySnapshot>({
    status: "loading",
    query: {},
    shown: null,
  });
  let latest = 0;

  const shown = (): HistoryResult | null => {
    const current = state.get();
    if (current.status === "ready") {
      return { query: current.query, page: current.page };
    }
    return current.status === "loading" ? current.shown : null;
  };

  return {
    state,
    async load(query) {
      latest += 1;
      const mine = latest;
      state.set({ status: "loading", query, shown: shown() });
      const result = report(await read(query));
      if (mine !== latest) return;
      state.set(
        result.ok
          ? { status: "ready", query, page: result.value }
          : { status: "failed", query, failure: result.failure },
      );
    },
  };
}
