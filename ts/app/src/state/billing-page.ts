// A billing page of one workspace, such as Plan and usage or Plans. The page
// reads again on an interval while Stripe confirms a payment or a billing
// operation runs, so the server's outcome replaces the wait by itself. A
// failed refresh keeps the last good page and shows the failure. A newer
// read always wins over an older one.

import type { ApiFailure, ApiResult } from "../api/client.ts";
import type { PlanStatus } from "../api/billing-types.ts";
import { createPoller, type Timers, type Visibility } from "./poller.ts";
import { createStore, type Store } from "./store.ts";

export type PageSnapshot<T> =
  | { status: "loading" }
  | { status: "ready"; page: T; refreshFailure: ApiFailure | null }
  | { status: "failed"; failure: ApiFailure };

export type PageStore<T> = {
  readonly state: Store<PageSnapshot<T>>;
  /** Read now. Returns whether the page still waits for the server. */
  load(): Promise<boolean>;
  /** Read once for this document; later calls share that read. */
  ensure(): Promise<boolean>;
  /** Read on the interval while the page waits. Returns the stop function. */
  watch(): () => void;
};

/** Whether a billing status waits for Stripe or for a running operation. */
export function statusWaits(status: PlanStatus): boolean {
  return status.payment_pending || status.operation_in_progress;
}

export const billingPollInterval = 5_000;

export function createPageStore<T>(options: {
  read: () => Promise<ApiResult<T>>;
  report: <R>(result: ApiResult<R>) => ApiResult<R>;
  waits: (page: T) => boolean;
  timers: Timers;
  visibility: Visibility;
}): PageStore<T> {
  const { read, report, waits } = options;
  const state = createStore<PageSnapshot<T>>({ status: "loading" });
  let latest = 0;
  let first: Promise<boolean> | null = null;

  const load = async (): Promise<boolean> => {
    latest += 1;
    const mine = latest;
    const result = report(await read());
    if (mine !== latest) {
      const current = state.get();
      return current.status === "ready" && waits(current.page);
    }
    if (result.ok) {
      state.set({ status: "ready", page: result.value, refreshFailure: null });
      return waits(result.value);
    }
    const current = state.get();
    state.set(
      current.status === "ready"
        ? { ...current, refreshFailure: result.failure }
        : { status: "failed", failure: result.failure },
    );
    return current.status === "ready" && waits(current.page);
  };

  const poller = createPoller(
    load,
    billingPollInterval,
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
