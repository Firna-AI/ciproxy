// The public site of ciproxy: the marketing pages at the root and the docs
// under /docs, on one host, ciproxy.dev. The cost claim of the landing page
// comes from cost-claim.ts, not from the CI of an organization.

/** The host of the marketing pages and the docs. */
export const siteHost = "ciproxy.dev";

/** The address of a page of the public site, for the screen `address`. */
export function siteAddress(page?: string): string {
  return page ? `${siteHost}/${page}` : siteHost;
}

/** The year in the footer of the site. */
export const copyrightYear = 2026;
