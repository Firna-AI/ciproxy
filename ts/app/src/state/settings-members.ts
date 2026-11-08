// The members of one workspace and the requests for access, as GET members
// gives them. A role change, an access grant, a removal, or a declined
// request runs through the session, and a success reads the members, the
// settings with the tab count, and the session again, because the change can
// end the role of the person who made it.

import type { ApiResult } from "../api/client.ts";
import type { Success, WorkspaceRole } from "../api/schema.ts";
import type { SettingsEndpoints } from "../api/settings-endpoints.ts";
import type { MembersView } from "../api/settings-schema.ts";
import type { Mutate, ReadSnapshot, Report } from "./settings-store.ts";
import { createStore, type Store } from "./store.ts";

export type MembersSnapshot = ReadSnapshot<MembersView>;

export type MembersStore = {
  readonly state: Store<MembersSnapshot>;
  load(): Promise<void>;
  ensure(): Promise<void>;
  /** Give a role to a request, or change the role of a member. */
  saveRole(memberId: string, role: WorkspaceRole): Promise<ApiResult<Success>>;
  /** Remove a member, or decline a request. */
  remove(memberId: string): Promise<ApiResult<Success>>;
};

export function createMembersStore(deps: {
  endpoints: SettingsEndpoints;
  workspaceId: string;
  mutate: Mutate;
  report: Report;
  /** Read the settings and the session again after a change. */
  afterChange: () => Promise<unknown>;
}): MembersStore {
  const { endpoints, workspaceId, mutate, report } = deps;
  const state = createStore<MembersSnapshot>({ status: "loading" });
  let first: Promise<void> | null = null;
  let latest = 0;

  /** Only the latest read sets the view, so an older answer cannot win. */
  const load = async (): Promise<void> => {
    latest += 1;
    const mine = latest;
    const result = report(await endpoints.members(workspaceId));
    if (mine !== latest) return;
    if (result.ok) {
      state.set({ status: "ready", value: result.value, refreshFailure: null });
      return;
    }
    const current = state.get();
    state.set(
      current.status === "ready"
        ? { ...current, refreshFailure: result.failure }
        : { status: "failed", failure: result.failure },
    );
  };

  const change = async (
    run: () => Promise<ApiResult<Success>>,
  ): Promise<ApiResult<Success>> => {
    const result = await mutate(run);
    if (result.ok) await Promise.all([load(), deps.afterChange()]);
    return result;
  };

  return {
    state,
    load,
    ensure() {
      first ??= load();
      return first;
    },
    saveRole: (memberId, role) =>
      change(() => endpoints.saveRole(workspaceId, memberId, role)),
    remove: (memberId) =>
      change(() => endpoints.removeMember(workspaceId, memberId)),
  };
}
