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
//
// Each organization of the Organizations tab is linked or removed. A linked
// one belongs to the workspace and counts for the rule that keeps the last
// organization, also while it is inactive. In the what-if of 20 November in
// access.ts, an owner removes the App from mokly-labs on GitHub, so the tab
// keeps it as removed for 30 days, beside mokly-ai.

import { labsRemoval } from "./access.ts";
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

/**
 * An organization in the Organizations tab, in one of two states. A linked
 * organization belongs to the workspace: active, or inactive while it has a
 * hold. A removed one left the workspace when an owner removed the App on
 * GitHub, and the tab keeps it for 30 days, with the day before which a new
 * install keeps its job history. The state decides the row, its actions, and
 * whether it counts for the rule of the last organization.
 */
export type WorkspaceOrganization = LinkedOrganization | RemovedOrganization;

/** An organization of the workspace, inactive while it has a hold. */
export type LinkedOrganization = OrganizationAccess & {
  state: "linked";
  holds?: readonly OrganizationHold[];
};

/**
 * An organization whose owner removed the App on GitHub. ciproxy has no
 * GitHub access left to it, so its row has no repositories and no Take out.
 */
export type RemovedOrganization = {
  state: "removed";
  organization: Organization;
  /** The day before which a new install keeps its job history. */
  installBefore: string;
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
  plan: { state: "linked", organization: studioOrg, holds: [planHold] },
  both: {
    state: "linked",
    organization: studioOrg,
    holds: [planHold, adminHold],
  },
  admin: { state: "linked", organization: studioOrg, holds: [adminHold] },
  active: { state: "linked", organization: studioOrg },
};

/** The two organizations of November, both linked and active. */
export const linkedOrganizations: readonly LinkedOrganization[] =
  organizationAccess.map((access) => ({ ...access, state: "linked" }));

/** The organizations of mokly-ai in December: the two of November, then one. */
export function withStudio(
  link: LinkedOrganization,
): readonly LinkedOrganization[] {
  return [...linkedOrganizations, link];
}

/** mokly-labs after an owner removed the App on GitHub on 20 November. */
export const removedLabs: RemovedOrganization = {
  state: "removed",
  organization: labsRemoval.organization,
  installBefore: labsRemoval.installBefore,
};

/** mokly-ai still linked, beside the removed mokly-labs. */
export const afterLabsRemoval: readonly WorkspaceOrganization[] = [
  ...linkedOrganizations.filter(
    (link) => link.organization.login !== labsRemoval.organization.login,
  ),
  removedLabs,
];

/**
 * The organizations that the rule of the last organization counts: each
 * linked one, also an inactive or a suspended one. A removed organization
 * left the workspace, so it does not count.
 */
export function linkedCount(list: readonly WorkspaceOrganization[]): number {
  return list.filter((row) => row.state === "linked").length;
}

/** Take out shows on a linked organization while another linked one stays. */
export function canTakeOut(
  row: WorkspaceOrganization,
  list: readonly WorkspaceOrganization[],
): boolean {
  return row.state === "linked" && linkedCount(list) > 1;
}

/** The organizations that mokly-ai holds from 8 December, inactive ones too. */
export const decemberOrganizationCount = linkedCount(
  withStudio(studioLinks.active),
);
