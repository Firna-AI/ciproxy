// The settings of one workspace: its name, labels, organizations, and the
// count of open requests, as GET settings gives them. A failed read keeps the
// last good view and shows the failure. Each change runs through the session,
// which repeats it once after a rotated CSRF token, and a change that
// succeeds reads the settings and the session again, so the tabs and the
// shell show the result. Only the latest read sets the view, so an older
// answer cannot bring back a row that a change removed.

import type { ApiFailure, ApiResult } from "../api/client.ts";
import type {
  LabelFit,
  LabelInput,
  LabelView,
  RepositorySelection,
  Success,
} from "../api/schema.ts";
import type { SettingsEndpoints } from "../api/settings-endpoints.ts";
import type { SettingsView } from "../api/settings-schema.ts";
import { createStore, type Store } from "./store.ts";

/** A read that a tab shows: loading, the last good value, or a failure. */
export type ReadSnapshot<T> =
  | { status: "loading" }
  | { status: "ready"; value: T; refreshFailure: ApiFailure | null }
  | { status: "failed"; failure: ApiFailure };

export type SettingsSnapshot = ReadSnapshot<SettingsView>;

export type Mutate = <T>(
  run: () => Promise<ApiResult<T>>,
) => Promise<ApiResult<T>>;

export type Report = <T>(result: ApiResult<T>) => ApiResult<T>;

export type SettingsDeps = {
  endpoints: SettingsEndpoints;
  workspaceId: string;
  mutate: Mutate;
  report: Report;
  /** Read the session again, for the name and the banner of the shell. */
  refreshSession: () => Promise<unknown>;
};

export type SettingsStore = {
  readonly state: Store<SettingsSnapshot>;
  /** Read the settings now. */
  load(): Promise<void>;
  /** Read once for this document; later calls share that read. */
  ensure(): Promise<void>;
  rename(name: string): Promise<ApiResult<Success>>;
  /** Add a label, or save the size of one by its ID. */
  saveLabel(
    labelId: string | null,
    input: LabelInput,
  ): Promise<ApiResult<LabelView>>;
  deleteLabel(labelId: string): Promise<ApiResult<Success>>;
  previewLabel(input: LabelInput): Promise<ApiResult<LabelFit>>;
  saveRepositories(
    organizationId: string,
    selection: RepositorySelection,
  ): Promise<ApiResult<Success>>;
  takeOut(organizationId: string): Promise<ApiResult<Success>>;
  turnOn(organizationId: string): Promise<ApiResult<Success>>;
};

export function createSettingsStore(deps: SettingsDeps): SettingsStore {
  const { endpoints, workspaceId, mutate, report } = deps;
  const state = createStore<SettingsSnapshot>({ status: "loading" });
  let first: Promise<void> | null = null;
  let latest = 0;

  /** Only the latest read sets the view, so an older answer cannot win. */
  const load = async (): Promise<void> => {
    latest += 1;
    const mine = latest;
    const result = report(await endpoints.settings(workspaceId));
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

  /** Run a change, then read again when it succeeds. */
  const change = async <T>(
    run: () => Promise<ApiResult<T>>,
    session: boolean,
  ): Promise<ApiResult<T>> => {
    const result = await mutate(run);
    if (result.ok) {
      await Promise.all([load(), session ? deps.refreshSession() : null]);
    }
    return result;
  };

  return {
    state,
    load,
    ensure() {
      first ??= load();
      return first;
    },
    rename: (name) => change(() => endpoints.rename(workspaceId, name), true),
    saveLabel: (labelId, input) =>
      change(
        () =>
          labelId === null
            ? endpoints.createLabel(workspaceId, input)
            : endpoints.updateLabel(workspaceId, labelId, input),
        false,
      ),
    deleteLabel: (labelId) =>
      change(() => endpoints.deleteLabel(workspaceId, labelId), false),
    previewLabel: (input) =>
      mutate(() => endpoints.previewLabel(workspaceId, input)),
    saveRepositories: (organizationId, selection) =>
      change(
        () =>
          endpoints.saveRepositories(workspaceId, organizationId, selection),
        false,
      ),
    takeOut: (organizationId) =>
      change(() => endpoints.takeOut(workspaceId, organizationId), true),
    turnOn: (organizationId) =>
      change(() => endpoints.turnOn(workspaceId, organizationId), true),
  };
}
