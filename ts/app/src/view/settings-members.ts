// The Members tab: the people with a role, and the requests for access that
// wait for an Admin. A person with no role yet is a request. Each person
// shows the name from GitHub with the login under it. A member shows the day
// of the role, and a request the day it asked and the organizations of the
// workspace that the person was a member of then. An Admin gives a role,
// changes one, or removes a person. Before a person gets the Admin role, a
// dialog shows the trust notice of the API, because an Admin controls the
// runners of every organization of the workspace.

import type { ApiFailure } from "../api/client.ts";
import type { WorkspaceRole } from "../api/schema.ts";
import type { MemberView, MembersView } from "../api/settings-schema.ts";
import type { FailureCopy } from "./claim.ts";
import { formatPlainDay, joinWords, roleNames } from "./format.ts";

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

/**
 * The name of a person: the name from GitHub, else the login, else the
 * GitHub ID before a sign-in.
 */
export function personName(member: MemberView): string {
  return member.name ?? member.login ?? `GitHub user ${member.id}`;
}

/** The login under a name, when the row shows a name in its place. */
function personCaption(member: MemberView): string | null {
  const login = member.login ?? null;
  return (member.name ?? null) === null ? null : login;
}

/**
 * The letters of an avatar: "CM" for the name Cal Moore, "M" for the login
 * mira, and "?" for a person known only by ID.
 */
function personInitials(member: MemberView): string {
  const name = member.name ?? null;
  if (name !== null) {
    const words = name
      .trim()
      .split(/\s+/)
      .filter((word) => word !== "");
    const letters = words.length > 1 ? [words[0], words.at(-1)] : words;
    const initials = letters.map((word) => word?.[0]?.toUpperCase() ?? "");
    if (initials.join("") !== "") return initials.join("");
  }
  return (member.login?.trim()[0] ?? "?").toUpperCase();
}

export type Person = {
  id: string;
  name: string;
  caption: string | null;
  initials: string;
};

export type MemberRow = Person & {
  you: boolean;
  role: WorkspaceRole;
  /** "1 September": the day of the role, or null when not known. */
  since: string | null;
};

export type RequestRow = Person & { asked: string };

/** "Asked on 18 November · member of mokly-ai": the request and its context. */
function askedText(member: MemberView, timeZone?: string): string {
  const asked = `Asked on ${formatPlainDay(member.requested_at, timeZone)}`;
  const organizations = member.request_organizations ?? [];
  if (organizations.length === 0) return asked;
  const logins = organizations.map((organization) => organization.login);
  return `${asked}\u00a0· member of ${joinWords(logins, "and")}`;
}

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
    const person = {
      id: member.id,
      name: personName(member),
      caption: personCaption(member),
      initials: personInitials(member),
    };
    const role = member.role ?? null;
    if (role === null) {
      requests.push({ ...person, asked: askedText(member, timeZone) });
    } else {
      const granted = member.granted_at ?? null;
      members.push({
        ...person,
        you: member.login === login,
        role,
        since: granted === null ? null : formatPlainDay(granted, timeZone),
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
