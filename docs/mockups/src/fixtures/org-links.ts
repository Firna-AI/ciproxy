// The what-ifs of Add an organization: the return from GitHub, and the
// organizations that join a workspace inactive. GitHub picks the organization.
// ciproxy then checks the install with the person's own GitHub sign-in and
// lists the workspaces where that person is an Admin: one links at once,
// several wait for a pick, and none waits until the person is an Admin again.
// A request to the owners waits for the approval and that check, and only the
// person who asked chooses. The workspace where Add started never picks one.
//
// On 2 November, Cal Moore is also an Admin of driftwood, a workspace on Free,
// so the return lists two workspaces. On 8 December, mokly-ai is on Free with
// mokly-ai and mokly-labs, which keep running, and Cal adds mokly-studio: Free
// includes one organization, so it joins inactive until Team. In a second
// what-if, Ana Souza adds it that day. After the check of the install,
// mokly-ai, her one Admin workspace, becomes its workspace. Her Admin role
// ends before ciproxy finishes the link, so mokly-studio joins on 8 December
// with two holds, which ciproxy records when it finishes the link. The move
// to Team on 11 December clears only the plan hold.

import { unbroken } from "./format.ts";
import { secondAdmin, type Member } from "./members.ts";
import { workspace, type Organization, type WorkspaceChoice } from "./org.ts";
import { organizationAccess, type OrganizationAccess } from "./settings.ts";
import { tiers } from "./tiers.ts";

/** mokly-ai on 2 November, before mokly-labs joins. */
const moklyBefore: WorkspaceChoice = {
  ...workspace,
  tier: "team",
  organizations: 1,
};

/** The second workspace where Cal Moore is an Admin, in the picker what-if. */
export const driftwood: WorkspaceChoice = {
  name: "driftwood",
  initial: "D",
  tier: "free",
  organizations: 1,
};

/** The workspaces where Cal is an Admin, in the order of the switch. */
export const adminWorkspaces: readonly WorkspaceChoice[] = [
  moklyBefore,
  driftwood,
];

/** The workspace that Cal picks for mokly-labs. */
export const pickedWorkspace = moklyBefore;

/**
 * A new organization joins inactive when the plan of the workspace includes
 * no more organizations. Every linked organization counts, also an inactive
 * one, and the organizations that a workspace kept on Free stay active.
 */
export function joinsInactive(choice: WorkspaceChoice): boolean {
  const limit = tiers[choice.tier].organizations;
  return limit !== undefined && choice.organizations >= limit;
}

/**
 * Ana Souza asks the owners of mokly-labs for the install. When an owner
 * approves it, she is an Admin of no workspace, so it waits for her.
 */
export const noAdminRequester: Member = secondAdmin;

/** The third organization, which joins mokly-ai on 8 December, on Free. */
export const studioOrg = {
  login: "mokly-studio",
  initial: "S",
  repositories: 4,
  publicRepositories: 1,
  joined: unbroken("8 December"),
} as const satisfies Organization;

/**
 * Why a linked organization is inactive, as ciproxy records it when it
 * finishes the link: the plan includes no more organizations, or the person
 * who added it lost the Admin role after the choice of its workspace. The two
 * holds clear on their own: the move to Team clears the plan hold, and a
 * current Admin turns the organization on.
 */
export type OrganizationHold =
  { kind: "plan" } | { kind: "admin"; addedBy: Member };

/** An organization of the workspace, inactive while it has a hold. */
export type LinkedOrganization = OrganizationAccess & {
  holds?: readonly OrganizationHold[];
};

const planHold: OrganizationHold = { kind: "plan" };

/**
 * Ana Souza adds mokly-studio on 8 December. Her Admin role ends after
 * mokly-ai becomes its workspace and before ciproxy finishes the link.
 */
export const formerAdmin: Member = secondAdmin;

const adminHold: OrganizationHold = { kind: "admin", addedBy: formerAdmin };

/** mokly-studio in each December what-if. */
export const studioLinks: Readonly<
  Record<"plan" | "both" | "admin" | "active", LinkedOrganization>
> = {
  plan: { organization: studioOrg, holds: [planHold] },
  both: { organization: studioOrg, holds: [planHold, adminHold] },
  admin: { organization: studioOrg, holds: [adminHold] },
  active: { organization: studioOrg },
};

/** The organizations of mokly-ai in December: the two of November, then one. */
export function withStudio(
  link: LinkedOrganization,
): readonly LinkedOrganization[] {
  return [...organizationAccess, link];
}

/** The organizations that mokly-ai holds from 8 December, inactive ones too. */
export const decemberOrganizationCount = withStudio(studioLinks.active).length;
