// The public site of ciproxy: the marketing pages at the root and the docs
// under /docs, on one host, ciproxy.dev. The landing page uses the CI of
// mokly-ai as its worked example, from the same 30 days as the savings
// report.

import { org } from "./org.ts";
import { savings } from "./savings.ts";

/** The host of the marketing pages and the docs. */
export const siteHost = "ciproxy.dev";

/** The address of a page of the public site, for the screen `address`. */
export function siteAddress(page?: string): string {
  return page ? `${siteHost}/${page}` : siteHost;
}

/** The Linux CI of mokly-ai in the 30 days of the savings report. */
export const reference = {
  org: org.login,
  jobs: savings.starts.own + savings.starts.boat,
  /** The most jobs that ran at the same time. */
  peakJobs: 92,
} as const;

/** The year in the footer of the site. */
export const copyrightYear = 2026;
