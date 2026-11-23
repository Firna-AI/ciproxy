// Skip for now on the provider step of Setup. The API keeps no record of a
// skip, so the skip is a memory of this browser tab: it lasts through a
// reload, belongs to one workspace and to the person who skipped, and ends
// with the tab. A connected provider always counts before a skip.

import { storageKeys, type KeyValueStorage } from "./storage.ts";
import { createStore, type Store } from "./store.ts";

export type ProviderSkipStore = {
  /** The login that skipped the step in this tab, or null. */
  readonly state: Store<string | null>;
  skip(login: string): void;
};

export function createProviderSkipStore(
  tab: KeyValueStorage,
  workspaceId: string,
): ProviderSkipStore {
  const key = `${storageKeys.providerSkipped}:${workspaceId}`;
  const state = createStore<string | null>(tab.get(key));
  return {
    state,
    skip(login) {
      tab.set(key, login);
      state.set(login);
    },
  };
}
