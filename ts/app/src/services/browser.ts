// The browser side of the services: fetch, the location, Web Storage, the
// clock, timers, and page visibility. The static render of the export runs
// without a window, so each part checks for one and stays inert without it.

import type { LinkElement, PageRouter } from "../state/navigation.ts";
import { memoryStorage, webStorage } from "../state/storage.ts";
import type { AppEnvironment } from "./services.ts";

const inertLocation = {
  origin: "",
  assign: () => undefined,
  replace: () => undefined,
  reload: () => undefined,
};

export function browserEnvironment(router: PageRouter): AppEnvironment {
  const browser = typeof window !== "undefined";
  return {
    transport: (url, init) => fetch(url, init),
    location: browser ? window.location : inertLocation,
    router,
    local: browser ? webStorage(() => window.localStorage) : memoryStorage(),
    tab: browser ? webStorage(() => window.sessionStorage) : memoryStorage(),
    now: () => Date.now(),
    page: () =>
      browser ? window.location.pathname + window.location.search : "",
    timers: {
      schedule(callback, delay) {
        const handle = setTimeout(callback, delay);
        return () => clearTimeout(handle);
      },
    },
    visibility: {
      hidden: () =>
        typeof document !== "undefined" &&
        document.visibilityState === "hidden",
      watch(listener) {
        if (typeof document === "undefined") return () => undefined;
        document.addEventListener("visibilitychange", listener);
        return () => document.removeEventListener("visibilitychange", listener);
      },
    },
  };
}

/** The link that a click target sits in, for the navigation gate. */
export function linkOf(target: unknown): LinkElement | null {
  if (!(target instanceof Element)) return null;
  const link = target.closest("a[href]");
  return link instanceof HTMLAnchorElement ? link : null;
}
