// The legal pages of the public site: the terms of service and the privacy
// policy, with their section headings. The user writes or approves the text
// of each section before the launch. Only the data retention of the privacy
// policy has text now, because security.md decides its limits.

import { appRemoval } from "./access.ts";
import { tierOrder, tiers } from "./tiers.ts";

export type LegalPageKey = "terms" | "privacy";

/** The legal pages in the order of the switch between them. */
export const legalPageOrder: readonly LegalPageKey[] = ["terms", "privacy"];

/** A section of a legal page: its id for "On this page", and its heading. */
export type LegalSection = { id: string; title: string };

export const legalPages: Readonly<
  Record<LegalPageKey, { title: string; sections: readonly LegalSection[] }>
> = {
  terms: {
    title: "Terms of service",
    sections: [
      { id: "service", title: "The service" },
      { id: "accounts", title: "Accounts, workspaces, and organizations" },
      { id: "plans", title: "Plans and payment" },
      { id: "machines", title: "Your machines and Boat" },
      { id: "use", title: "Acceptable use" },
      { id: "data", title: "Your data" },
      { id: "agent", title: "The open-source agent" },
      { id: "ending", title: "Ending the service" },
      { id: "liability", title: "Warranties and liability" },
      { id: "changes", title: "Changes to these terms" },
      { id: "contact", title: "Contact" },
    ],
  },
  privacy: {
    title: "Privacy policy",
    sections: [
      { id: "who", title: "Who we are" },
      { id: "stored", title: "What ciproxy stores" },
      { id: "never", title: "What ciproxy never receives" },
      { id: "use", title: "How ciproxy uses data" },
      { id: "processors", title: "Who processes data for ciproxy" },
      { id: "retention", title: "Data retention" },
      { id: "rights", title: "Your rights" },
      { id: "security", title: "Security" },
      { id: "changes", title: "Changes to this policy" },
      { id: "contact", title: "Contact" },
    ],
  },
};

/** The section of the privacy policy that states the decided limits. */
export const retentionId = "retention";

/** The limits of "Data retention" in security.md, in days. */
const keep = { jobDays: 90, totalsDays: 90, cacheHitDays: 7 } as const;

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
    kept: `${appRemoval.keepDays} days after the removal`,
  },
];

/** "7 days on Free, 30 days on Team, and 90 days on Business". */
export function historyWindows(): string {
  const parts = tierOrder.map(
    (key) => `${tiers[key].history} on ${tiers[key].name}`,
  );
  const last = parts.pop();
  return `${parts.join(", ")}, and ${last ?? ""}`;
}
