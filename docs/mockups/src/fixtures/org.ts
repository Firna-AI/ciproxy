// The sample workspace of the mockups. A workspace is the customer of
// ciproxy: it holds one or more GitHub organizations and owns the tier, the
// members, the machines, the labels, the settings, and the Boat account.
// mokly-ai signs up on Tuesday 1 September 2026, so the workspace takes its
// name from that organization. It passes the Free starts on Friday 4
// September, moves to Team that day, and runs three own machines from
// mid-September. On Monday 2 November it adds a second organization,
// mokly-labs. The November screens show a busy Team workspace; the December
// screens show the payment problems.

import { unbroken } from "./format.ts";
import type { Role } from "./members.ts";

/** The dashboard host shown in the address bar of every screen. */
export const host = "app.ciproxy.dev";

/** A GitHub organization of the workspace, with its private repositories. */
export type Organization = {
  login: string;
  initial: string;
  repositories: number;
  publicRepositories: number;
  /** The day the organization joined the workspace. */
  joined: string;
};

/** The first organization of the workspace, which installed ciproxy. */
export const org = {
  login: "mokly-ai",
  initial: "M",
  repositories: 14,
  publicRepositories: 2,
  joined: unbroken("1 September"),
} as const satisfies Organization;

/** The second organization, which an Admin added on 2 November. */
export const labsOrg = {
  login: "mokly-labs",
  initial: "L",
  repositories: 5,
  publicRepositories: 0,
  joined: unbroken("2 November"),
} as const satisfies Organization;

/** The organizations of the workspace, the first one first. */
export const organizations: readonly Organization[] = [org, labsOrg];

/** The workspace, which the switcher of the top bar shows. */
export const workspace = { name: org.login, initial: org.initial } as const;

/** The name that an Admin types in the rename dialog of the workspace. */
export const newWorkspaceName = "Mokly";

/** A workspace in the switcher of the top bar. */
export type WorkspaceChoice = {
  name: string;
  initial: string;
  tier: TierKey;
  organizations: number;
};

/** A workspace in the switch of the top bar, with the role of the person. */
export type SwitchChoice = WorkspaceChoice & { role: Role };

/** The workspaces of Cal Moore: Admin of mokly-ai, and Viewer of tidepool. */
export const workspaceChoices: readonly SwitchChoice[] = [
  {
    ...workspace,
    tier: "team",
    organizations: organizations.length,
    role: "admin",
  },
  {
    name: "tidepool",
    initial: "T",
    tier: "free",
    organizations: 1,
    role: "viewer",
  },
];

/** "1 organization", "2 organizations". */
export function organizationsText(count: number): string {
  return `${count} ${count === 1 ? "organization" : "organizations"}`;
}

/** "mokly-ai or mokly-labs", the organizations of the workspace in a line. */
export function organizationList(word: "and" | "or"): string {
  const logins = organizations.map((organization) => organization.login);
  const last = logins.pop() ?? "";
  return logins.length > 0 ? `${logins.join(", ")} ${word} ${last}` : last;
}

export const user = {
  name: "Cal Moore",
  login: "calummoore",
  initials: "CM",
} as const;

/**
 * The address of a dashboard page of the workspace, for the screen
 * `address`. The workspace takes its address from its first organization.
 */
export function orgAddress(page: string): string {
  return `${host}/${workspace.name}/${page}`;
}

/** The tiers the shell can show beside the workspace name. */
export type TierKey = "free" | "team" | "business";
