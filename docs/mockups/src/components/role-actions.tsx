// What a person sees in place of an action that the role does not allow: the
// words that name who can act. Everyone sees the state of the organization,
// and only the roles that may act on it get its actions. A Billing person
// pays; only an Admin changes the machines, the labels, Boat, and the roles.

import type { Role } from "../fixtures/members.ts";

/** An Admin or a Billing person pays; a Viewer does not. */
export function canPay(role: Role): boolean {
  return role !== "viewer";
}

/** In place of a payment action, such as Update card or Choose Team. */
export function AskPayer() {
  return <span className="caption">Ask an Admin or a Billing person</span>;
}

/** In place of an action that only an Admin takes, such as a setting. */
export function AskAdmin() {
  return <span className="caption">Ask an Admin</span>;
}
