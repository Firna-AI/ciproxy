// The addresses of the site and of the dashboard. ciproxy.dev serves the
// marketing pages at the root and the docs under /docs/. The control plane
// serves the dashboard on its one host, app.ciproxy.dev.

/** The dashboard, on the one host of the control plane. */
export const appOrigin = "https://app.ciproxy.dev";

/**
 * The sign-in page of the dashboard. Every "Start free" button and the
 * "Sign in" link open it, because every workspace starts on Free.
 */
export const signInUrl = `${appOrigin}/sign-in`;

/** The pages of the site that other pages link to. */
export const sitePages = {
  home: "/",
  pricing: "/pricing/",
  docs: "/docs/",
  terms: "/terms/",
  privacy: "/privacy/",
} as const;

/** The part of the site that a page belongs to, for the top bar. */
export type SiteSection = "home" | "pricing" | "legal" | "docs";

/** The year of the copyright line: the year of the build. */
export const copyrightYear = new Date().getFullYear();
