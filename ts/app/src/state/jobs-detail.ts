// One job of a workspace, as GET jobs/{job_id} gives it. The read passes
// through the session report. A job outside the plan window answers 404,
// which the page shows as its own state.

import type { ApiFailure, ApiResult } from "../api/client.ts";
import type { JobDetailPage } from "../api/jobs-types.ts";
import { createStore, type Store } from "./store.ts";

export type JobSnapshot =
  | { status: "loading" }
  | { status: "ready"; page: JobDetailPage }
  | { status: "failed"; failure: ApiFailure };

export type JobStore = {
  readonly state: Store<JobSnapshot>;
  load(): Promise<void>;
};

export function createJobStore(options: {
  read: () => Promise<ApiResult<JobDetailPage>>;
  report: <T>(result: ApiResult<T>) => ApiResult<T>;
}): JobStore {
  const { read, report } = options;
  const state = createStore<JobSnapshot>({ status: "loading" });
  let latest = 0;
  return {
    state,
    async load() {
      latest += 1;
      const mine = latest;
      if (state.get().status === "failed") state.set({ status: "loading" });
      const result = report(await read());
      if (mine !== latest) return;
      state.set(
        result.ok
          ? { status: "ready", page: result.value }
          : { status: "failed", failure: result.failure },
      );
    },
  };
}
