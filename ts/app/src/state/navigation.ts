// Page changes. Normally the router changes the page in place. After the API
// build changed, the next page change is a full-document navigation, so the
// new page loads the new dashboard before it reads the API. The gate covers
// the three ways a page changes: a navigation in code, a click on an app
// link, and a move through the browser history.

import { apiPrefix } from "../api/client.ts";
import type { BuildGate } from "./build-gate.ts";

export type PageRouter = {
  push(href: string): void;
  replace(href: string): void;
};

export type DocumentLocation = {
  readonly origin: string;
  assign(href: string): void;
  replace(href: string): void;
  reload(): void;
};

export type Navigator = {
  /** Open an app page. */
  go(href: string, options?: { replace?: boolean }): void;
  /** Leave the app for a top-level URL, such as GitHub or an API route. */
  leave(href: string): void;
};

export function createNavigator(
  gate: BuildGate,
  location: DocumentLocation,
  router: PageRouter,
): Navigator {
  return {
    go(href, options) {
      const replace = options?.replace === true;
      if (gate.changed()) {
        gate.beginReload();
        if (replace) location.replace(href);
        else location.assign(href);
      } else if (replace) {
        router.replace(href);
      } else {
        router.push(href);
      }
    },
    leave(href) {
      location.assign(href);
    },
  };
}

/** The parts of a link that decide whether the app may handle a click. */
export type LinkElement = {
  readonly href: string;
  readonly target: string;
  hasAttribute(name: string): boolean;
};

export type ClickEvent = {
  readonly button: number;
  readonly metaKey: boolean;
  readonly ctrlKey: boolean;
  readonly shiftKey: boolean;
  readonly altKey: boolean;
  readonly defaultPrevented: boolean;
  preventDefault(): void;
  stopPropagation(): void;
};

/**
 * The page that a plain click on a link opens inside the app, or null when
 * an action handles it or the browser handles it: a modified click, another
 * target, a download, another origin, or an API route.
 */
export function appLinkHref(
  link: LinkElement | null,
  event: ClickEvent,
  origin: string,
): string | null {
  if (
    link === null ||
    event.defaultPrevented ||
    event.button !== 0 ||
    link.hasAttribute("data-ciproxy-action")
  ) {
    return null;
  }
  if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) {
    return null;
  }
  if (!["", "_self"].includes(link.target) || link.hasAttribute("download")) {
    return null;
  }
  let url: URL;
  try {
    url = new URL(link.href, origin);
  } catch {
    return null;
  }
  if (url.origin !== origin) return null;
  if (url.pathname === apiPrefix || url.pathname.startsWith(`${apiPrefix}/`)) {
    return null;
  }
  return url.pathname + url.search + url.hash;
}

export type GateWindow = {
  readonly location: DocumentLocation;
  addEventListener(
    type: "popstate",
    listener: (event: { stopImmediatePropagation(): void }) => void,
    capture: boolean,
  ): void;
  removeEventListener(
    type: "popstate",
    listener: (event: { stopImmediatePropagation(): void }) => void,
    capture: boolean,
  ): void;
};

export type GateDocument = {
  addEventListener(
    type: "click",
    listener: (event: ClickEvent & { readonly target: unknown }) => void,
    capture: boolean,
  ): void;
  removeEventListener(
    type: "click",
    listener: (event: ClickEvent & { readonly target: unknown }) => void,
    capture: boolean,
  ): void;
};

/**
 * Listen before the router: in the capture phase, a click on an app link and
 * a history move become full-document navigations once the build changed.
 * Returns the function that removes the listeners.
 */
export function installNavigationGate(
  gate: BuildGate,
  win: GateWindow,
  doc: GateDocument,
  linkOf: (target: unknown) => LinkElement | null,
): () => void {
  const onClick = (event: ClickEvent & { readonly target: unknown }) => {
    if (!gate.changed()) return;
    const href = appLinkHref(linkOf(event.target), event, win.location.origin);
    if (href === null) return;
    event.preventDefault();
    event.stopPropagation();
    gate.beginReload();
    win.location.assign(href);
  };
  const onHistory = (event: { stopImmediatePropagation(): void }) => {
    if (!gate.changed()) return;
    event.stopImmediatePropagation();
    gate.beginReload();
    win.location.reload();
  };
  doc.addEventListener("click", onClick, true);
  win.addEventListener("popstate", onHistory, true);
  return () => {
    doc.removeEventListener("click", onClick, true);
    win.removeEventListener("popstate", onHistory, true);
  };
}
