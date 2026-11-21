// The live queue of one workspace: GET jobs/live, read again on an interval
// while the page shows. A failed refresh keeps the last queue and shows the
// failure. Every read passes through the session report, so a 401 ends the
// signed-in view. Only the newest read can change the view.

import type { ApiFailure, ApiResult } from "../api/client.ts";
import type { JobsLivePage } from "../api/jobs-types.ts";
import { createPoller, type Timers, type Visibility } from "./poller.ts";
import { createStore, type Store } from "./store.ts";

export type LiveSnapshot =
  | { status: "loading" }
  | {
      status: "ready";
      page: JobsLivePage;
      /** The clock time of the read, for the time that each job waits. */
      readAt: number;
      refreshFailure: ApiFailure | null;
    }
  | { status: "failed"; failure: ApiFailure };

export type LiveStore = {
  readonly state: Store<LiveSnapshot>;
  /** Read the queue now. */
  load(): Promise<void>;
  /** Read on the interval while the tab shows. Returns the stop function. */
  watch(): () => void;
};

export const livePollInterval = 5_000;

export function createLiveStore(options: {
  read: () => Promise<ApiResult<JobsLivePage>>;
  report: <T>(result: ApiResult<T>) => ApiResult<T>;
  now: () => number;
  timers: Timers;
  visibility: Visibility;
}): LiveStore {
  const { read, report, now } = options;
  const state = createStore<LiveSnapshot>({ status: "loading" });
  let latest = 0;

  const load = async (): Promise<void> => {
    latest += 1;
    const mine = latest;
    const result = report(await read());
    if (mine !== latest) return;
    if (result.ok) {
      state.set({
        status: "ready",
        page: result.value,
        readAt: now(),
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

  const poller = createPoller(
    async () => {
      await load();
      return true;
    },
    livePollInterval,
    options.timers,
    options.visibility,
  );

  return {
    state,
    load,
    watch() {
      poller.start();
      return () => poller.stop();
    },
  };
}
