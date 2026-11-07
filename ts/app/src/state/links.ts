// The person's own installation links on the return page of Add an
// organization. Each link keeps its own pick, request, and failure, so two
// adds at once stay independent. A background read never drops a pick that
// the person made, while its workspace is still offered.

import type { ApiFailure } from "../api/client.ts";
import { failureStatus } from "../api/client.ts";
import type { Endpoints } from "../api/endpoints.ts";
import type { LinkState, OrganizationLinkView } from "../api/schema.ts";
import { createPoller, type Timers, type Visibility } from "./poller.ts";
import type { SessionStore } from "./session.ts";
import { createStore, type Store } from "./store.ts";

export type LinkEntry = {
  link: OrganizationLinkView;
  /** The workspace that the person picked, before the add. */
  picked: string | null;
  choosing: boolean;
  failure: ApiFailure | null;
};

export type LinksSnapshot = {
  /** The links of this page, in the order in which they first showed. */
  entries: readonly LinkEntry[];
  checking: boolean;
  checkFailure: ApiFailure | null;
};

const finalStates: ReadonlySet<LinkState> = new Set([
  "linked",
  "linked_inactive",
]);
const movingStates: ReadonlySet<LinkState> = new Set([
  "confirming_installation",
  "linking",
]);

/** A link that still needs the person, GitHub, or ciproxy. */
export function isOpenLink(link: OrganizationLinkView): boolean {
  return !finalStates.has(link.state);
}

/**
 * A link that joins the return page: one that is still open, or one that the
 * exact claims of the latest sign-in name, so that its outcome shows.
 */
export function joinsReturnPage(
  link: OrganizationLinkView,
  named: readonly string[],
): boolean {
  return isOpenLink(link) || named.includes(link.id);
}

/** A link that changes by itself, so the page reads it again. */
export function isMovingLink(link: OrganizationLinkView): boolean {
  return movingStates.has(link.state) || link.activation?.preparing === true;
}

export const linkPollInterval = 3_000;

export type LinksStore = {
  readonly state: Store<LinksSnapshot>;
  /**
   * Merge links read elsewhere, such as the links of the session. A new link
   * joins the page when it joins the return page with the `named` claims.
   */
  seed(links: readonly OrganizationLinkView[], named?: readonly string[]): void;
  pick(claimId: string, workspaceId: string): void;
  choose(claimId: string): Promise<void>;
  /** Check the links again with fresh GitHub proof, then the session. */
  checkAgain(): Promise<void>;
  /** Read the moving links on the interval. Returns the stop function. */
  watch(): () => void;
};

export function createLinksStore(options: {
  endpoints: Endpoints;
  session: SessionStore;
  timers: Timers;
  visibility: Visibility;
}): LinksStore {
  const { endpoints, session } = options;
  const state = createStore<LinksSnapshot>({
    entries: [],
    checking: false,
    checkFailure: null,
  });

  const update = (claimId: string, change: (entry: LinkEntry) => LinkEntry) => {
    const current = state.get();
    state.set({
      ...current,
      entries: current.entries.map((entry) =>
        entry.link.id === claimId ? change(entry) : entry,
      ),
    });
  };

  let watching = false;
  /** Poll again while a page watches, for a link that changes by itself. */
  const resume = () => {
    if (
      watching &&
      state.get().entries.some((entry) => isMovingLink(entry.link))
    ) {
      poller.start();
    }
  };

  const withLink = (
    entry: LinkEntry,
    link: OrganizationLinkView,
  ): LinkEntry => {
    const offered = link.workspaces.some(
      (choice) => choice.id === entry.picked,
    );
    return { ...entry, link, picked: offered ? entry.picked : null };
  };

  const seed = (
    links: readonly OrganizationLinkView[],
    named: readonly string[] = [],
  ) => {
    const current = state.get();
    const known = new Map(
      current.entries.map((entry) => [entry.link.id, entry]),
    );
    const entries = current.entries.map((entry) => {
      const next = links.find((link) => link.id === entry.link.id);
      return next === undefined ? entry : withLink(entry, next);
    });
    for (const link of links) {
      if (!known.has(link.id) && joinsReturnPage(link, named)) {
        entries.push({ link, picked: null, choosing: false, failure: null });
      }
    }
    state.set({ ...current, entries });
    resume();
  };

  /** Read one link again. A failed read shows on its entry. */
  const reread = async (claimId: string, keep: ApiFailure | null = null) => {
    const result = session.report(await endpoints.link(claimId));
    update(claimId, (entry) =>
      result.ok
        ? { ...withLink(entry, result.value), failure: keep }
        : { ...entry, failure: keep ?? result.failure },
    );
  };

  const poll = async (): Promise<boolean> => {
    const moving = state
      .get()
      .entries.filter((entry) => isMovingLink(entry.link));
    await Promise.all(moving.map((entry) => reread(entry.link.id)));
    return state.get().entries.some((entry) => isMovingLink(entry.link));
  };

  const poller = createPoller(
    poll,
    linkPollInterval,
    options.timers,
    options.visibility,
  );

  return {
    state,
    seed,
    pick(claimId, workspaceId) {
      update(claimId, (entry) => ({
        ...entry,
        picked: workspaceId,
        failure: null,
      }));
    },
    async choose(claimId) {
      const entry = state
        .get()
        .entries.find((item) => item.link.id === claimId);
      if (entry === undefined || entry.choosing || entry.picked === null)
        return;
      const workspaceId = entry.picked;
      update(claimId, (item) => ({ ...item, choosing: true, failure: null }));
      const result = await session.mutate(() =>
        endpoints.chooseWorkspace(claimId, workspaceId),
      );
      if (result.ok) {
        update(claimId, (item) => ({
          ...withLink(item, result.value),
          choosing: false,
        }));
        resume();
        return;
      }
      update(claimId, (item) => ({
        ...item,
        choosing: false,
        failure: result.failure,
      }));
      const status = failureStatus(result.failure);
      if (status === 403 || status === 409)
        await reread(claimId, result.failure);
    },
    async checkAgain() {
      if (state.get().checking) return;
      state.set({ ...state.get(), checking: true, checkFailure: null });
      const result = session.report(await endpoints.links());
      if (result.ok) seed(result.value);
      state.set({
        ...state.get(),
        checking: false,
        checkFailure: result.ok ? null : result.failure,
      });
      if (result.ok) await session.refresh();
      resume();
    },
    watch() {
      watching = true;
      resume();
      return () => {
        watching = false;
        poller.stop();
      };
    },
  };
}
