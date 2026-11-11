// How long a sign-in lasts, and what happens on GitHub in the screens of an
// ended access: Tom Becker, an owner of mokly-ai, suspends or removes the
// ciproxy GitHub App, an owner of mokly-labs removes it from mokly-labs alone,
// and Jonas Weber, a Viewer, leaves mokly-ai on GitHub. Each is a what-if of
// November, not a step of the main story.

import { formatDay, unbroken } from "./format.ts";
import { owner, viewer } from "./members.ts";
import { labsOrg } from "./org.ts";
import { teamPeriodEnd } from "./usage.ts";

/** A sign-in lasts 30 days, and ciproxy checks the membership each hour. */
export const signIn = { days: 30, membershipCheckMinutes: 60 } as const;

/** The days that ciproxy keeps the data of an organization after a removal. */
const keepDays = 30;

/** Tom Becker removes the App on Friday 20 November 2026, in UTC. */
const removedOn = new Date(Date.UTC(2026, 10, 20));

/** The day on which ciproxy deletes the data of mokly-ai. */
const deletedOn = new Date(removedOn.getTime() + keepDays * 86_400_000);

export const appRemoval = {
  by: owner.name,
  on: formatDay(removedOn),
  keepDays,
  deletedOn: formatDay(deletedOn),
  /** Team ends at the end of the paid period, as a move to Free does. */
  planEnds: teamPeriodEnd,
} as const;

/**
 * An owner of mokly-labs removes the App from mokly-labs alone, on the same
 * Friday 20 November. mokly-labs leaves the workspace. mokly-ai keeps its
 * jobs, and the workspace keeps its machines, its Boat account, and its
 * subscription. ciproxy keeps the data of mokly-labs for the same 30 days,
 * so a new install before the day of `appRemoval` keeps its job history.
 */
export const labsRemoval = {
  organization: labsOrg,
  installBefore: appRemoval.deletedOn,
} as const;

/** Tom Becker suspends the App on Thursday 19 November. */
export const appSuspension = {
  by: owner.name,
  on: unbroken("19 November"),
} as const;

/** Jonas Weber left mokly-ai on GitHub, so his Viewer role ended. */
export const leaver = viewer;
