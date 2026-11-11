// The repositories that the GitHub App can see in one organization, as
// GET repositories gives them. The page reads them when it opens, and Try
// again reads them again. A save goes through the settings store, which
// holds the repository access of each organization.

import type { ApiFailure, ApiResult } from "../api/client.ts";
import type { RepositoryView } from "../api/settings-schema.ts";
import type { Report } from "./settings-store.ts";
import { createStore, type Store } from "./store.ts";

export type RepositoriesSnapshot =
  | { status: "loading" }
  | { status: "ready"; repositories: RepositoryView[] }
  | { status: "failed"; failure: ApiFailure };

export type RepositoriesStore = {
  readonly state: Store<RepositoriesSnapshot>;
  load(): Promise<void>;
};

export function createRepositoriesStore(options: {
  read: () => Promise<ApiResult<RepositoryView[]>>;
  report: Report;
}): RepositoriesStore {
  const state = createStore<RepositoriesSnapshot>({ status: "loading" });
  let latest = 0;
  return {
    state,
    async load() {
      latest += 1;
      const mine = latest;
      if (state.get().status === "failed") state.set({ status: "loading" });
      const result = options.report(await options.read());
      if (mine !== latest) return;
      state.set(
        result.ok
          ? { status: "ready", repositories: result.value }
          : { status: "failed", failure: result.failure },
      );
    },
  };
}
