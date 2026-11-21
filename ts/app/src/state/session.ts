// The signed-in person, as GET /api/session gives it. The store loads the
// session once per document, refreshes it on request, and keeps the last good
// session through a failed refresh, with the failure visible. A 401 from any
// call ends the signed-in view. A response that a sign-out or an end
// overtook never restores the view, so a stale request cannot bring private
// data back.

import {
  failureStatus,
  signInRequired,
  type ApiFailure,
  type ApiResult,
} from "../api/client.ts";
import type { Endpoints } from "../api/endpoints.ts";
import type { SessionView, Success } from "../api/schema.ts";
import { storageKeys, type KeyValueStorage } from "./storage.ts";
import { createStore, type Store } from "./store.ts";

/** Why sign-in shows: the 30-day sign-in ended, or nobody is signed in. */
export type SignedOutReason = "ended" | "none";

export type SessionSnapshot =
  | { status: "loading" }
  | {
      status: "signed-in";
      session: SessionView;
      refreshFailure: ApiFailure | null;
    }
  | { status: "signed-out"; reason: SignedOutReason }
  | { status: "failed"; failure: ApiFailure };

export type SessionStore = {
  readonly state: Store<SessionSnapshot>;
  /** Load the session once; later calls share the first load. */
  ensure(): Promise<SessionSnapshot>;
  /** Read the session again, keeping the current view on a failure. */
  refresh(): Promise<SessionSnapshot>;
  /**
   * Check the session for a page change. The first visit loads it. A later
   * visit reads it again, unless a read just finished, so a role that a
   * membership check ended shows on the next page.
   */
  visit(): Promise<SessionSnapshot>;
  /** The CSRF token of the current session, or null before one loads. */
  csrf(): string | null;
  /**
   * Run a change. A 403 can mean that another tab rotated the CSRF token,
   * so the store refreshes the session once and repeats the change only when
   * the token changed. A refused change had no effect, so the repeat cannot
   * apply it twice.
   */
  mutate<T>(run: () => Promise<ApiResult<T>>): Promise<ApiResult<T>>;
  /**
   * Pass the result of a read through. A 401 ends the signed-in view, and a
   * 403 reads the session again, because the role may have ended.
   */
  report<T>(result: ApiResult<T>): ApiResult<T>;
  signOut(): Promise<ApiResult<Success>>;
  claimAdmin(workspaceId: string): Promise<ApiResult<Success>>;
};

/** A read that finished this recently answers a page change too. */
export const sessionFreshFor = 2_000;

export function createSessionStore(
  endpoints: () => Endpoints,
  storage: KeyValueStorage,
  now: () => number,
): SessionStore {
  const state = createStore<SessionSnapshot>({ status: "loading" });
  let generation = 0;
  let pending: Promise<SessionSnapshot> | null = null;
  let readAt: number | null = null;

  const signedOut = (): SessionSnapshot => {
    const ends = Date.parse(storage.get(storageKeys.signInEnds) ?? "");
    const reason = Number.isFinite(ends) && now() >= ends ? "ended" : "none";
    return { status: "signed-out", reason };
  };

  const end = (): void => {
    generation += 1;
    pending = null;
    state.set(signedOut());
  };

  const load = (): Promise<SessionSnapshot> => {
    if (pending !== null) return pending;
    const mine = generation;
    const request = endpoints()
      .session()
      .then((result): SessionSnapshot => {
        if (mine !== generation) return state.get();
        readAt = now();
        if (result.ok) {
          storage.set(storageKeys.signInEnds, result.value.expires_at);
          state.set({
            status: "signed-in",
            session: result.value,
            refreshFailure: null,
          });
        } else if (signInRequired(result.failure)) {
          end();
        } else {
          const current = state.get();
          state.set(
            current.status === "signed-in"
              ? { ...current, refreshFailure: result.failure }
              : { status: "failed", failure: result.failure },
          );
        }
        return state.get();
      })
      .finally(() => {
        if (pending === request) pending = null;
      });
    pending = request;
    return request;
  };

  const csrf = (): string | null => {
    const current = state.get();
    return current.status === "signed-in" ? current.session.csrf_token : null;
  };

  const settle = <T>(result: ApiResult<T>): ApiResult<T> => {
    if (!result.ok && signInRequired(result.failure)) end();
    return result;
  };

  const report = <T>(result: ApiResult<T>): ApiResult<T> => {
    settle(result);
    if (!result.ok && failureStatus(result.failure) === 403) void load();
    return result;
  };

  const mutate = async <T>(
    run: () => Promise<ApiResult<T>>,
  ): Promise<ApiResult<T>> => {
    const first = await run();
    if (first.ok || failureStatus(first.failure) !== 403) return settle(first);
    const before = csrf();
    const after = await load();
    if (after.status !== "signed-in" || csrf() === before) return first;
    return settle(await run());
  };

  let loaded: Promise<SessionSnapshot> | null = null;
  const ensure = (): Promise<SessionSnapshot> => {
    loaded ??= load();
    return loaded;
  };

  return {
    state,
    ensure,
    refresh: load,
    visit() {
      if (loaded === null) return ensure();
      if (pending !== null) return pending;
      if (readAt !== null && now() - readAt < sessionFreshFor) {
        return Promise.resolve(state.get());
      }
      return load();
    },
    csrf,
    mutate,
    report,
    async signOut() {
      const result = await mutate(() => endpoints().signOut());
      if (result.ok) {
        storage.remove(storageKeys.signInEnds);
        generation += 1;
        pending = null;
        state.set({ status: "signed-out", reason: "none" });
      }
      return result;
    },
    async claimAdmin(workspaceId) {
      const result = await mutate(() => endpoints().claimAdmin(workspaceId));
      if (result.ok) await load();
      return result;
    },
  };
}
