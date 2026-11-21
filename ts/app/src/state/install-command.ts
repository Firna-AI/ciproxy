// The one-line install command of a workspace. Each command holds a secret
// that adds machines for one hour, so the store creates one only on request,
// at most one at a time, and never on a background refresh. A second request
// while one runs, or while a command is open, reuses it.

import type { ApiFailure, ApiResult } from "../api/client.ts";
import type { Endpoints } from "../api/endpoints.ts";
import type { InstallCommand, Success } from "../api/schema.ts";
import { createStore, type Store } from "./store.ts";

export type InstallCommandSnapshot =
  | { status: "none" }
  | { status: "creating" }
  | {
      status: "open";
      command: InstallCommand;
      revokeFailure: ApiFailure | null;
    }
  | { status: "revoking"; command: InstallCommand }
  | { status: "revoked" }
  | { status: "failed"; failure: ApiFailure };

export type InstallCommandStore = {
  readonly state: Store<InstallCommandSnapshot>;
  /** Create a command unless one is open or on its way. */
  create(): Promise<void>;
  revoke(): Promise<void>;
};

/** Whether an open command has passed its end. */
export function commandExpired(command: InstallCommand, now: number): boolean {
  return now >= Date.parse(command.expires_at);
}

export function createInstallCommandStore(options: {
  endpoints: Endpoints;
  workspaceId: string;
  mutate: <T>(run: () => Promise<ApiResult<T>>) => Promise<ApiResult<T>>;
  now: () => number;
}): InstallCommandStore {
  const { endpoints, workspaceId, mutate, now } = options;
  const state = createStore<InstallCommandSnapshot>({ status: "none" });

  const create = async (): Promise<void> => {
    const current = state.get();
    if (current.status === "creating" || current.status === "revoking") return;
    if (current.status === "open" && !commandExpired(current.command, now())) {
      return;
    }
    state.set({ status: "creating" });
    const result = await mutate(() =>
      endpoints.issueInstallCommand(workspaceId),
    );
    state.set(
      result.ok
        ? { status: "open", command: result.value, revokeFailure: null }
        : { status: "failed", failure: result.failure },
    );
  };

  const revoke = async (): Promise<void> => {
    const current = state.get();
    if (current.status !== "open") return;
    const { command } = current;
    state.set({ status: "revoking", command });
    const result: ApiResult<Success> = await mutate(() =>
      endpoints.revokeInstallCommand(workspaceId, command.id),
    );
    state.set(
      result.ok
        ? { status: "revoked" }
        : { status: "open", command, revokeFailure: result.failure },
    );
  };

  return { state, create, revoke };
}
