// The one banner of a workspace and scope. Native page responses supply the
// page banner; other pages and Organizations use GET /banner with brief
// reuse. A failed native read keeps the last banner and its page shows the
// failure. An endpoint failure has its own notice. Accepted responses share
// read-start order, so a newer accepted response wins. Native failures publish
// nothing and cannot suppress an older successful response.

import type { ApiFailure, ApiResult } from "../api/client.ts";
import type { PageBanner, PageBannerView } from "../api/billing-types.ts";
import { createStore, type Store } from "./store.ts";

export type BannerSnapshot =
  | { status: "loading" }
  | {
      status: "ready";
      banner: PageBanner | null;
      refreshFailure: ApiFailure | null;
    }
  | { status: "failed"; failure: ApiFailure };

export type BannerStore = {
  readonly state: Store<BannerSnapshot>;
  /** Read for a page change, unless a read is on its way or just finished. */
  visit(): Promise<void>;
  /** Retry the fallback endpoint now. */
  refresh(): Promise<void>;
  /** Publish a native page banner without changing its result or failure. */
  readPage<T>(
    read: () => Promise<ApiResult<T>>,
    select: (page: T) => PageBannerView["banner"],
  ): Promise<ApiResult<T>>;
};

/** A read that finished this recently answers a page change too. */
export const bannerFreshFor = 2_000;

export function createBannerStore(options: {
  read: () => Promise<ApiResult<PageBannerView>>;
  report: <T>(result: ApiResult<T>) => ApiResult<T>;
  now: () => number;
}): BannerStore {
  const { read, report, now } = options;
  const state = createStore<BannerSnapshot>({ status: "loading" });
  let latest = 0;
  let accepted = 0;
  let pending: Promise<void> | null = null;
  let readAt: number | null = null;

  const publish = (banner: PageBannerView["banner"]): void => {
    readAt = now();
    state.set({
      status: "ready",
      banner: banner ?? null,
      refreshFailure: null,
    });
  };

  const refresh = (): Promise<void> => {
    latest += 1;
    const mine = latest;
    const request = read()
      .then(report)
      .then((result) => {
        if (mine <= accepted) return;
        accepted = mine;
        if (result.ok) {
          publish(result.value.banner);
          return;
        }
        readAt = now();
        const current = state.get();
        state.set(
          current.status === "ready"
            ? { ...current, refreshFailure: result.failure }
            : { status: "failed", failure: result.failure },
        );
      })
      .finally(() => {
        if (pending === request) pending = null;
      });
    pending = request;
    return request;
  };

  return {
    state,
    refresh,
    async readPage(read, select) {
      latest += 1;
      const mine = latest;
      // An older endpoint cannot answer a later fallback visit after this read.
      pending = null;
      const result = await read();
      if (result.ok && mine > accepted) {
        accepted = mine;
        publish(select(result.value));
      }
      return result;
    },
    visit() {
      if (pending !== null) return pending;
      if (readAt !== null && now() - readAt < bannerFreshFor) {
        return Promise.resolve();
      }
      return refresh();
    },
  };
}
