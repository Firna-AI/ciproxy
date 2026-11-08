// The one banner of a workspace, as GET /banner gives it. Each page change
// reads it again, unless a read just finished, and a billing change reads it
// at once. A failed read keeps the last banner and shows the failure; a
// failed first read shows the failure alone, so the page never hides a
// payment problem behind silence. A newer read always wins over an older one.

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
  /** Read now, such as after a payment or a plan change. */
  refresh(): Promise<void>;
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
  let pending: Promise<void> | null = null;
  let readAt: number | null = null;

  const refresh = (): Promise<void> => {
    latest += 1;
    const mine = latest;
    const request = read()
      .then(report)
      .then((result) => {
        if (mine !== latest) return;
        readAt = now();
        if (result.ok) {
          state.set({
            status: "ready",
            banner: result.value.banner ?? null,
            refreshFailure: null,
          });
          return;
        }
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
    visit() {
      if (pending !== null) return pending;
      if (readAt !== null && now() - readAt < bannerFreshFor) {
        return Promise.resolve();
      }
      return refresh();
    },
  };
}
