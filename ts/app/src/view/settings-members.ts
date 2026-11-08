// The Members tab: the people with a role, and the requests for access that
// wait for an Admin. A person with no role yet is a request. An Admin gives
// a role, changes one, or removes a person. Before a person gets the Admin
// role, a dialog shows the trust notice of the API, because an Admin
// controls the runners of every organization of the workspace.

import type { ApiFailure } from "../api/client.ts";
import type { WorkspaceRole } from "../api/schema.ts";
import type { MemberView, MembersView } from "../api/settings-schema.ts";
import type { FailureCopy } from "./claim.ts";
import { roleNames } from "./format.ts";

/** The roles in the order of the role picker, with what each may do. */
export const roleOrder: readonly WorkspaceRole[] = [
  "admin",
  "billing",
  "viewer",
];

export const roleDescriptions: Readonly<Record<WorkspaceRole, string>> = {
  admin:
    "Can do everything: the machines, labels, settings, Boat, roles, and billing.",
  billing:
    "Sees what a Viewer sees, and manages the plan, the add-ons, and the invoices.",
  viewer: "Sees the jobs, the machines, the savings, and Plan and usage.",
};

export const roleOptions = roleOrder.map((role) => ({
  label: roleNames[role],
  value: role,
}));

/** The role that a request gets unless the Admin picks another. */
export const requestRole: WorkspaceRole = "viewer";

/** "1 September", a day in the time zone of the browser. */
export function formatDay(iso: string, timeZone?: string): string {
  return new Intl.DateTimeFormat("en-GB", {
    day: "numeric",
    month: "long",
    timeZone,
  }).format(new Date(iso));
}

/** The name of a person: the login, or the GitHub ID before a sign-in. */
export function personName(member: MemberView): string {
  return member.login ?? `GitHub user ${member.id}`;
}

export type MemberRow = {
  id: string;
  name: string;
  you: boolean;
  role: WorkspaceRole;
  since: string;
};

export type RequestRow = { id: string; name: string; asked: string };

export type MembersModel = {
  requests: RequestRow[];
  members: MemberRow[];
};

/** The members and the requests, in the order of the API. */
export function membersModel(
  view: MembersView,
  login: string,
  timeZone?: string,
): MembersModel {
  const requests: RequestRow[] = [];
  const members: MemberRow[] = [];
  for (const member of view.members) {
    const name = personName(member);
    const day = formatDay(member.requested_at, timeZone);
    const role = member.role ?? null;
    if (role === null) {
      requests.push({ id: member.id, name, asked: `Asked on ${day}` });
    } else {
      members.push({
        id: member.id,
        name,
        you: member.login === login,
        role,
        since: day,
      });
    }
  }
  return { requests, members };
}

/** What a pick of a role does: nothing, a save, or the Admin dialog first. */
export function roleStep(
  current: WorkspaceRole | null,
  next: WorkspaceRole,
): "none" | "save" | "confirm-admin" {
  if (current === next) return "none";
  return next === "admin" ? "confirm-admin" : "save";
}

export type MemberAction = "role" | "grant" | "remove" | "decline";

const titles: Readonly<Record<MemberAction, string>> = {
  role: "Unable to change the role",
  grant: "Unable to give access",
  remove: "Unable to remove the person",
  decline: "Unable to decline the request",
};

export function memberFailure(
  failure: ApiFailure,
  action: MemberAction,
): FailureCopy {
  return {
    title: titles[action],
    body:
      failure.kind === "network"
        ? "Check the connection, then try again."
        : "ciproxy could not finish the change. Try again.",
  };
}
