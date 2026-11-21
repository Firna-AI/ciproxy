// The build of the API that this document talks to. Every API response names
// its build. The first named build is the baseline. A later response with
// another build latches `changed`, and nothing unlatches it, also a late
// response from the old build. The navigation gate then turns the next page
// change into a full-document navigation, so this tab never calls the newer
// API with old types, and the current page keeps its input until then.
//
// The gate also notes the page at the latch. When the page then changed by
// a way that the navigation gate did not see first, such as a history move
// that the router handled, `leftPage` tells the API client, which reloads
// the document in place of the first read of the new page.

export type BuildGate = {
  /** Record the build of one API response. Absent values are ignored. */
  observe(build: string | null): void;
  /** Whether a response named a build other than the baseline. */
  changed(): boolean;
  /** Whether a full-document navigation has started. */
  reloading(): boolean;
  /** Mark the start of a full-document navigation. API calls stop. */
  beginReload(): void;
  /** Whether the page changed after the build changed. */
  leftPage(): boolean;
};

/**
 * `page` names the current page, such as its path and query. Without it, the
 * gate never reports a page change.
 */
export function createBuildGate(page?: () => string): BuildGate {
  let baseline: string | null = null;
  let changed = false;
  let reloading = false;
  let latchedPage: string | null = null;
  return {
    observe(build) {
      const value = build?.trim() ?? "";
      if (value === "") return;
      if (baseline === null) {
        baseline = value;
      } else if (value !== baseline && !changed) {
        changed = true;
        latchedPage = page?.() ?? null;
      }
    },
    changed: () => changed,
    reloading: () => reloading,
    beginReload() {
      reloading = true;
    },
    leftPage: () =>
      changed &&
      latchedPage !== null &&
      page !== undefined &&
      page() !== latchedPage,
  };
}
