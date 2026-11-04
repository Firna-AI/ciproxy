// The people of mokly-ai in ciproxy and their roles. Each role contains the
// one below it. Cal Moore installed the GitHub App on 1 September, so he is
// the first Admin. Lena Fischer is a member of mokly-ai on GitHub with no
// role yet: her sign-in asks the Admins for access. Tom Becker is an owner of
// mokly-ai on GitHub, so he can claim the Admin role.

import { unbroken } from "./format.ts";
import { user } from "./org.ts";

export type Role = "admin" | "billing" | "viewer";

/** The roles in the order of the role picker. */
export const roleOrder: readonly Role[] = ["admin", "billing", "viewer"];

/** The name of each role and what it may do. */
export const roles: Readonly<Record<Role, { name: string; can: string }>> = {
  admin: {
    name: "Admin",
    can: "Can do everything: the machines, labels, settings, Boat, roles, and billing.",
  },
  billing: {
    name: "Billing",
    can: "Sees what a Viewer sees, and manages the plan, the add-ons, and the invoices.",
  },
  viewer: {
    name: "Viewer",
    can: "Sees the jobs, the machines, the savings, and Plan and usage.",
  },
};

export type Member = {
  name: string;
  login: string;
  initials: string;
  role: Role;
  /** The day the person got the role. */
  since: string;
  /** How the person got it, when not from an Admin. */
  note?: string;
};

/** Jonas Weber, a Viewer, whose view the read-only billing screens show. */
export const viewer: Member = {
  name: "Jonas Weber",
  login: "jweber",
  initials: "JW",
  role: "viewer",
  since: unbroken("21 September"),
};

export const members: readonly Member[] = [
  {
    ...user,
    role: "admin",
    since: unbroken("1 September"),
    note: "installed the GitHub App",
  },
  {
    name: "Ana Souza",
    login: "anasouza",
    initials: "AS",
    role: "admin",
    since: unbroken("2 September"),
  },
  {
    name: "Dana Kim",
    login: "danakim",
    initials: "DK",
    role: "billing",
    since: unbroken("4 September"),
  },
  viewer,
];

/** The member of mokly-ai on GitHub who has no role and asks for access. */
export const requester = {
  name: "Lena Fischer",
  login: "lfischer",
  initials: "LF",
  asked: unbroken("18 November"),
} as const;

/** An owner of mokly-ai on GitHub who has no role in ciproxy yet. */
export const owner = {
  name: "Tom Becker",
  login: "tbecker",
  initials: "TB",
} as const;
