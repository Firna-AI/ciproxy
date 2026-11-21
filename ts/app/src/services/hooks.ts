// Hooks that connect screens to the stores. Every hook reads the latest
// value at the time it acts, so no handler keeps a stale session or token.

import { useCallback, useEffect, useSyncExternalStore } from "react";

import type { SessionView } from "../api/schema.ts";
import { signInHref } from "../routing/pages.ts";
import type { SessionSnapshot } from "../state/session.ts";
import type { Store } from "../state/store.ts";
import { checkAgainTarget } from "../view/access.ts";
import { useServices } from "./context.tsx";

export function useValue<T>(store: Store<T>): T {
  return useSyncExternalStore(store.subscribe, store.get, store.get);
}

/**
 * The session of a page that needs one. Each page change checks it again,
 * so a role that ended shows at once. A signed-out session opens sign-in,
 * with the notice when the sign-in ended. `page` names the page and its
 * workspace, such as "setup:<id>".
 */
export function useSession(page: string): SessionSnapshot {
  const { session, navigator } = useServices();
  const snapshot = useValue(session.state);
  useEffect(() => {
    void session.visit();
  }, [session, page]);
  useEffect(() => {
    if (snapshot.status !== "signed-out") return;
    navigator.go(signInHref(snapshot.reason === "ended" ? "ended" : null), {
      replace: true,
    });
  }, [snapshot, navigator]);
  return snapshot;
}

/** Open another page once, when `href` is set. */
export function useRedirect(href: string | null): void {
  const { navigator } = useServices();
  useEffect(() => {
    if (href !== null) navigator.go(href, { replace: true });
  }, [href, navigator]);
}

/**
 * Check again: fresh GitHub proof for a sign-in without a workspace session,
 * or a new read of the links and the session for any other sign-in.
 */
export function useCheckAgain(): (session: SessionView) => Promise<void> {
  const services = useServices();
  return useCallback(
    async (session: SessionView) => {
      const target = checkAgainTarget(session);
      if ("leave" in target) {
        services.navigator.leave(target.leave);
        return;
      }
      await services.session.refresh();
    },
    [services],
  );
}
