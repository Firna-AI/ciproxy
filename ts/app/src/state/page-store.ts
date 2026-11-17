// A page that one read gives, such as Plan and usage, Plans, or the overflow
// summary of Machines. A failed refresh keeps the last good page and shows
// the failure. A newer read always wins over an older one. A page store can
// also read again on an interval while a screen watches it and the server's
// outcome is still to come, so that outcome replaces the wait by itself.

import type { ApiFailure, ApiResult } from "../api/client.ts";
import { createPoller, type Timers, type Visibility } from "./poller.ts";
import { createStore, type Store } from "./store.ts";

export type PageSnapshot<T> =
  | { status: "loading" }
  | { status: "ready"; page: T; refreshFailure: ApiFailure | null }
  | { status: "failed"; failure: ApiFailure };

/** The reads of one page, with no interval. */
export type PageRead<T> = {
  readonly state: Store<PageSnapshot<T>>;
  /** Read now. A later read makes this one count for nothing. */
  load(): Promise<void>;
  /** Read once for this document; later calls share that read. */
  ensure(): Promise<void>;
};

export type PageStore<T> = PageRead<T> & {
  /**
   * Read on the interval while the page waits, or while no good page exists.
   * Returns the stop function.
   */
  watch(): () => void;
};

/** The reads of one page, for a page that never reads on an interval. */
export function createPageRead<T>(options: {
  read: () => Promise<ApiResult<T>>;
  report: <R>(result: ApiResult<R>) => ApiResult<R>;
}): PageRead<T> {
  const { read, report } = options;
  const state = createStore<PageSnapshot<T>>({ status: "loading" });
  let latest = 0;
  let first: Promise<void> | null = null;

  const load = async (): Promise<void> => {
    latest += 1;
    const mine = latest;
    const result = report(await read());
    if (mine !== latest) return;
    if (result.ok) {
      state.set({ status: "ready", page: result.value, refreshFailure: null });
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
    ensure() {
      first ??= load();
      return first;
    },
  };
}

/**
 * The reads of one page that a screen can watch. While watched, the store
 * reads every `interval` ms until a read leaves a good page that does not
 * wait. Each poll decides from the stored page, not from its own reply, so a
 * failed read with no good page keeps the poll going.
 */
export function createPageStore<T>(options: {
  read: () => Promise<ApiResult<T>>;
  report: <R>(result: ApiResult<R>) => ApiResult<R>;
  /** Whether a good page waits for the server, so the poll goes on. */
  waits: (page: T) => boolean;
  /** The time between two reads while a screen watches the page, in ms. */
  interval: number;
  timers: Timers;
  visibility: Visibility;
}): PageStore<T> {
  const { waits } = options;
  const page = createPageRead(options);

  const poll = async (): Promise<boolean> => {
    await page.load();
    const current = page.state.get();
    return current.status !== "ready" || waits(current.page);
  };

  const poller = createPoller(
    poll,
    options.interval,
    options.timers,
    options.visibility,
  );

  return {
    ...page,
    watch() {
      poller.start();
      return () => poller.stop();
    },
  };
}
