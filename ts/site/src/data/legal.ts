// The legal pages of the site. Retention limits follow
// docs/protocol/security.md, app-removal.md, and overflow.md.

import { listText } from "./format.ts";
import { privacySections } from "./legal-privacy.ts";
import { termsSections } from "./legal-terms.ts";
import { sitePages } from "./site.ts";
import { tierOrder, tiers } from "./tiers.ts";

export type LegalPageKey = "terms" | "privacy";

/** The legal pages in the order of the switch between them. */
export const legalPageOrder: readonly LegalPageKey[] = ["terms", "privacy"];

/** A legal section with plain-text paragraphs and a stable navigation id. */
export type LegalSection = {
  readonly id: string;
  readonly title: string;
  readonly paragraphs: readonly string[];
};

export type LegalPage = {
  title: string;
  /** The address of the page. */
  href: string;
  sections: readonly LegalSection[];
};

export const legalPages: Readonly<Record<LegalPageKey, LegalPage>> = {
  terms: {
    title: "Terms of service",
    href: sitePages.terms,
    sections: termsSections,
  },
  privacy: {
    title: "Privacy policy",
    href: sitePages.privacy,
    sections: privacySections,
  },
};

/** The section that also renders the retention table and plan history. */
export const retentionId = "retention";

/** The decided limits, in days. */
const keep = {
  jobDays: 90,
  totalsDays: 90,
  cacheHitDays: 7,
  /** The data of an organization after it removes the GitHub App. */
  removalDays: 30,
} as const;

export type RetentionRow = { data: string; kept: string };

export const retentionRows: readonly RetentionRow[] = [
  {
    data: "The details of each finished job",
    kept: `${keep.jobDays} days, on every plan`,
  },
  {
    data: "The daily totals of the savings report",
    kept: `${keep.totalsDays} days`,
  },
  { data: "The cache hits of each machine", kept: `${keep.cacheHitDays} days` },
  { data: "The last report of each machine", kept: "Until its next report" },
  {
    data: "The record of a person",
    kept: "While the person has a role or an open request for access",
  },
  {
    data: "The data of an organization that removes the GitHub App",
    kept: `${keep.removalDays} days after removal, with earlier normal expiry still in force`,
  },
  {
    data: "The workspace data after its last organization removes the GitHub App",
    kept: `${keep.removalDays} days after removal, with earlier normal expiry still in force`,
  },
  {
    data: "A Boat key after a key change, Disconnect, or removal of the last organization",
    kept: "Only until the last sandbox that needs that key ends",
  },
];

/** "7 days on Free, 30 days on Team, and 90 days on Business". */
export function historyWindows(): string {
  return listText(
    tierOrder.map((key) => `${tiers[key].history} on ${tiers[key].name}`),
  );
}
