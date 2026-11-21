// Skip for now on the Boat step of Setup. The API keeps no record of a skip,
// so the skip is a memory of this browser tab: it lasts through a reload,
// belongs to one workspace and to the person who skipped, and ends with the
// tab. A connected Boat account always counts before a skip.

import { storageKeys, type KeyValueStorage } from "./storage.ts";
import { createStore, type Store } from "./store.ts";

export type BoatSkipStore = {
  /** The login that skipped the step in this tab, or null. */
  readonly state: Store<string | null>;
  skip(login: string): void;
};

export function createBoatSkipStore(
  tab: KeyValueStorage,
  workspaceId: string,
): BoatSkipStore {
  const key = `${storageKeys.boatSkipped}:${workspaceId}`;
  const state = createStore<string | null>(tab.get(key));
  return {
    state,
    skip(login) {
      tab.set(key, login);
      state.set(login);
    },
  };
}
