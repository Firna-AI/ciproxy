// The return from Stripe Checkout for one exact session. The page asks the
// server at once, then again on an interval while Stripe confirms. After a
// few answers that still confirm, the page says so and keeps checking by
// itself. A busy answer, while a payment event holds the billing lock, counts
// as one more wait. Only the server's outcome ends the wait; any other
// failure stops and shows Try again.

import {
  failureStatus,
  type ApiFailure,
  type ApiResult,
} from "../api/client.ts";
import type {
  CheckoutReturnPage,
  PaymentOutcome,
} from "../api/billing-types.ts";
import { createPoller, type Timers, type Visibility } from "./poller.ts";
import { createStore, type Store } from "./store.ts";

export type ReturnSnapshot =
  | { status: "confirming"; attempts: number; still: boolean }
  | { status: "settled"; outcome: PaymentOutcome }
  | { status: "failed"; failure: ApiFailure; attempts: number };

export type ReturnStore = {
  readonly state: Store<ReturnSnapshot>;
  /** Ask the server now, then on the interval while Stripe confirms. */
  start(): void;
  /** Ask again after a failure. */
  retry(): void;
  stop(): void;
};

export const returnPollInterval = 3_000;

/** After this many answers that still confirm, the page says so. */
export const confirmingAnswers = 3;

export function createReturnStore(options: {
  reconcile: () => Promise<ApiResult<CheckoutReturnPage>>;
  mutate: <T>(run: () => Promise<ApiResult<T>>) => Promise<ApiResult<T>>;
  timers: Timers;
  visibility: Visibility;
  /** Read the changed workspace again: its pages and its banner. */
  settled: () => Promise<void>;
}): ReturnStore {
  const { reconcile, mutate, settled } = options;
  const state = createStore<ReturnSnapshot>({
    status: "confirming",
    attempts: 0,
    still: false,
  });
  /** Whether the page shows, so a check that waits starts the interval. */
  let active = false;
  /** The latest check that the page started itself, not on the interval. */
  let first: Promise<boolean> | null = null;

  const attempts = (): number => {
    const current = state.get();
    return current.status === "settled" ? 0 : current.attempts;
  };

  const wait = (): boolean => {
    const next = attempts() + 1;
    state.set({
      status: "confirming",
      attempts: next,
      still: next >= confirmingAnswers,
    });
    return true;
  };

  const check = async (): Promise<boolean> => {
    const result = await mutate(reconcile);
    if (!result.ok) {
      if (failureStatus(result.failure) === 409) return wait();
      state.set({
        status: "failed",
        failure: result.failure,
        attempts: attempts(),
      });
      return false;
    }
    const { outcome } = result.value;
    if (outcome.state === "confirming") return wait();
    state.set({ status: "settled", outcome });
    await settled();
    return false;
  };

  const poller = createPoller(
    check,
    returnPollInterval,
    options.timers,
    options.visibility,
  );

  /** Keep checking on the interval after a check that still waits. */
  const follow = (checking: Promise<boolean>): void => {
    void checking.then((again) => {
      if (again && active) poller.start();
    });
  };

  return {
    state,
    start() {
      if (active) return;
      active = true;
      if (state.get().status !== "confirming") return;
      first ??= check();
      follow(first);
    },
    retry() {
      if (state.get().status !== "failed") return;
      const count = attempts();
      state.set({
        status: "confirming",
        attempts: count,
        still: count >= confirmingAnswers,
      });
      first = check();
      follow(first);
    },
    stop() {
      active = false;
      poller.stop();
    },
  };
}
