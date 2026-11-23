// The overflow settings of one workspace, for Setup, Settings under Overflow,
// and the label dialog: the switch, the wait, and each provider with its
// account and its own spend limit. The store runs one operation at a time:
// each read and each change waits for the one before it, so no read can
// bring back a value that an accepted change replaced. A change holds the
// pending mark from its request through the read after it, and every form
// reads the mark. A policy save takes only the edit: when its turn comes,
// the store adds the latest confirmed values, so no form can send an old
// value that it did not mean to change. A change that the API accepted is
// never sent again: when the read after it fails, the store shows the failed
// read with the accepted change, and Try again only reads. A key check runs
// in the same order and holds the same mark, but it saves nothing, so no
// read follows it. The key of a check or a connection waits only inside its
// queued request, and the form drops it once the API accepted the
// connection.

import type { ApiFailure, ApiResult } from "../api/client.ts";
import type { OverflowEndpoints } from "../api/overflow-endpoints.ts";
import type {
  BoatConnection,
  BoatWallets,
  OverflowPolicy,
  OverflowProvider,
  OverflowSettings,
  ProviderSpendLimit,
  Success,
} from "../api/schema.ts";
import { createStore, type Store } from "./store.ts";

/** A change that the API accepted, with the provider that it changed. */
export type AcceptedChange =
  | { kind: "connected" | "disconnected"; provider: OverflowProvider }
  | { kind: "saved" };

export type OverflowSnapshot =
  | { status: "loading" }
  | {
      status: "ready";
      settings: OverflowSettings;
      refreshFailure: ApiFailure | null;
    }
  | { status: "failed"; failure: ApiFailure; accepted: AcceptedChange | null };

/** A new monthly spend limit of one provider. */
export type LimitEdit = ProviderSpendLimit & { provider: OverflowProvider };

/** The values of the policy that a save means to change. */
export type PolicyEdit = Partial<OverflowPolicy> | LimitEdit;

export type OverflowStore = {
  readonly state: Store<OverflowSnapshot>;
  /** Set from the request of a change to the end of the read after it. */
  readonly pending: Store<boolean>;
  /** Read the settings after the operations before it. */
  load(): Promise<void>;
  /** Read once for this document; later calls share that read. */
  ensure(): Promise<void>;
  /** Check a key of a provider and list its wallets. It saves nothing. */
  checkKey(
    provider: OverflowProvider,
    apiKey: string,
  ): Promise<ApiResult<BoatWallets>>;
  /**
   * Connect a key of a provider with its limit, and the wallet of a provider
   * that bills one of several. A refused change saves nothing. `onAccepted`
   * runs once the API accepted the key, before the new read, so the form can
   * drop the key at once. A provider with an account gets a key change.
   */
  connect(
    provider: OverflowProvider,
    request: BoatConnection,
    onAccepted?: () => void,
  ): Promise<ApiResult<Success>>;
  disconnect(provider: OverflowProvider): Promise<ApiResult<Success>>;
  /**
   * Save an edit of the policy, on the latest confirmed settings. Null when
   * the store sent nothing, because it knew no saved values to build on.
   */
  save(edit: PolicyEdit): Promise<ApiResult<Success> | null>;
};

/** The complete policy of an edit, or null when a value is unknown. */
function policyFrom(
  settings: OverflowSettings | null,
  edit: Partial<OverflowPolicy>,
): OverflowPolicy | null {
  const enabled = edit.enabled ?? settings?.enabled;
  const wait = edit.wait_seconds ?? settings?.wait_seconds;
  if (enabled === undefined || wait === undefined) return null;
  return { enabled, wait_seconds: wait };
}

/** Whether a provider has an account in the settings, also a rejected one. */
function hasAccount(
  settings: OverflowSettings | null,
  provider: OverflowProvider,
): boolean {
  return (
    settings?.providers.some(
      (item) => item.provider === provider && item.account != null,
    ) ?? false
  );
}

export function createOverflowStore(options: {
  endpoints: OverflowEndpoints;
  workspaceId: string;
  report: <T>(result: ApiResult<T>) => ApiResult<T>;
  mutate: <T>(run: () => Promise<ApiResult<T>>) => Promise<ApiResult<T>>;
}): OverflowStore {
  const { endpoints, workspaceId, report, mutate } = options;
  const state = createStore<OverflowSnapshot>({ status: "loading" });
  const pending = createStore(false);
  let tail: Promise<unknown> = Promise.resolve();
  let queuedLoad: Promise<void> | null = null;
  let changes = 0;
  let first: Promise<void> | null = null;

  const enqueue = <T>(task: () => Promise<T>): Promise<T> => {
    const run = tail.then(task);
    tail = run.catch(() => undefined);
    return run;
  };

  const read = async (accepted: AcceptedChange | null): Promise<void> => {
    const result = report(await endpoints.overflow(workspaceId));
    if (result.ok) {
      state.set({
        status: "ready",
        settings: result.value,
        refreshFailure: null,
      });
      return;
    }
    const current = state.get();
    if (accepted === null && current.status === "ready") {
      state.set({ ...current, refreshFailure: result.failure });
      return;
    }
    const kept = current.status === "failed" ? current.accepted : null;
    state.set({
      status: "failed",
      failure: result.failure,
      accepted: accepted ?? kept,
    });
  };

  const load = (): Promise<void> => {
    if (queuedLoad !== null) return queuedLoad;
    const request = enqueue(() => read(null)).finally(() => {
      if (queuedLoad === request) queuedLoad = null;
    });
    queuedLoad = request;
    return request;
  };

  /** Queue a change, and hold the pending mark until no change is left. */
  const queue = <T>(task: () => Promise<T>): Promise<T> => {
    queuedLoad = null;
    changes += 1;
    pending.set(true);
    return enqueue(task).finally(() => {
      changes -= 1;
      if (changes === 0) pending.set(false);
    });
  };

  const send = async <T extends Success>(
    accepted: AcceptedChange,
    request: () => Promise<ApiResult<T>>,
    onAccepted?: () => void,
  ): Promise<ApiResult<T>> => {
    const result = await mutate(request);
    if (!result.ok) return result;
    onAccepted?.();
    await read(accepted);
    return result;
  };

  const confirmed = (): OverflowSettings | null => {
    const current = state.get();
    return current.status === "ready" ? current.settings : null;
  };

  return {
    state,
    pending,
    load,
    ensure() {
      first ??= load();
      return first;
    },
    checkKey: (provider, apiKey) =>
      queue(() =>
        mutate(() => endpoints.checkKey(workspaceId, provider, apiKey)),
      ),
    connect: (provider, request, onAccepted) =>
      queue(() =>
        send(
          { kind: "connected", provider },
          () =>
            hasAccount(confirmed(), provider)
              ? endpoints.changeProviderKey(workspaceId, provider, request)
              : endpoints.connectProvider(workspaceId, provider, request),
          onAccepted,
        ),
      ),
    disconnect: (provider) =>
      queue(() =>
        send({ kind: "disconnected", provider }, () =>
          endpoints.disconnectProvider(workspaceId, provider),
        ),
      ),
    save: (edit) =>
      queue(async () => {
        const settings = confirmed();
        if ("provider" in edit) {
          if (settings === null) return null;
          const limit = { spend_limit_cents: edit.spend_limit_cents };
          return send({ kind: "saved" }, () =>
            endpoints.saveProviderLimit(workspaceId, edit.provider, limit),
          );
        }
        const policy = policyFrom(settings, edit);
        if (policy === null) return null;
        return send({ kind: "saved" }, () =>
          endpoints.saveOverflow(workspaceId, policy),
        );
      }),
  };
}
