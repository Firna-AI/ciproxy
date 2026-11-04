// The people of mokly-ai in ciproxy and their roles. Each role contains the
// one below it. Cal Moore installed the GitHub App on 1 September, so he is
// the first Admin. Ana Souza is an Admin whom GitHub does not let manage
// self-hosted runners, so she cannot add machines. Dana Kim has the Billing
// role and is an owner of mokly-ai on GitHub, so she can claim the Admin role
// from the account menu. Lena Fischer is a member of mokly-ai on GitHub with
// no role yet: her sign-in asks the Admins for access. Tom Becker is an owner
// of mokly-ai on GitHub with no role, so he can claim the Admin role at once.

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
  /** An owner of the organization on GitHub, who can claim the Admin role. */
  owner?: boolean;
};

/** Jonas Weber, a Viewer, whose view the read-only billing screens show. */
export const viewer: Member = {
  name: "Jonas Weber",
  login: "jweber",
  initials: "JW",
  role: "viewer",
  since: unbroken("21 September"),
};

/** Ana Souza, an Admin without the GitHub right to add self-hosted runners. */
export const adminWithoutRight: Member = {
  name: "Ana Souza",
  login: "anasouza",
  initials: "AS",
  role: "admin",
  since: unbroken("2 September"),
};

/** Dana Kim, who pays, and who is an owner of mokly-ai on GitHub. */
export const billingPerson: Member = {
  name: "Dana Kim",
  login: "danakim",
  initials: "DK",
  role: "billing",
  since: unbroken("4 September"),
  owner: true,
};

/** Cal Moore, who installed the GitHub App, so he is the first Admin. */
export const firstAdmin: Member = {
  ...user,
  role: "admin",
  since: unbroken("1 September"),
  note: "installed the GitHub App",
  owner: true,
};

export const members: readonly Member[] = [
  firstAdmin,
  adminWithoutRight,
  billingPerson,
  viewer,
];

/** The person whose view a screen of each role shows. */
export const personFor: Readonly<Record<Role, Member>> = {
  admin: firstAdmin,
  billing: billingPerson,
  viewer,
};

/** The member of mokly-ai on GitHub who has no role and asks for access. */
export const requester = {
  name: "Lena Fischer",
  login: "lfischer",
  initials: "LF",
  asked: unbroken("18 November"),
} as const;

/** The requests for access that wait for an Admin, from 18 November. */
export const openRequests: readonly (typeof requester)[] = [requester];

/** An owner of mokly-ai on GitHub who has no role in ciproxy yet. */
export const owner = {
  name: "Tom Becker",
  login: "tbecker",
  initials: "TB",
} as const;
