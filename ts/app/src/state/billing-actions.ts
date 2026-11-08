// A billing action that pays nothing: open the Stripe billing portal for the
// invoices and the card, or schedule Move to Free. One run at a time; a
// failure stays visible until the next run.

import type { ApiFailure, ApiResult } from "../api/client.ts";
import { createStore, type Store } from "./store.ts";

export type ActionSnapshot =
  | { status: "idle" }
  | { status: "working" }
  | { status: "done" }
  | { status: "failed"; failure: ApiFailure };

export type ActionStore = {
  readonly state: Store<ActionSnapshot>;
  run(): Promise<void>;
};

export function createActionStore<T>(options: {
  request: () => Promise<ApiResult<T>>;
  mutate: <R>(run: () => Promise<ApiResult<R>>) => Promise<ApiResult<R>>;
  /** What follows a success, such as leaving for the portal. */
  done: (value: T) => Promise<void> | void;
}): ActionStore {
  const { request, mutate, done } = options;
  const state = createStore<ActionSnapshot>({ status: "idle" });
  return {
    state,
    async run() {
      if (state.get().status === "working") return;
      state.set({ status: "working" });
      const result = await mutate(request);
      if (!result.ok) {
        state.set({ status: "failed", failure: result.failure });
        return;
      }
      state.set({ status: "done" });
      await done(result.value);
    },
  };
}
