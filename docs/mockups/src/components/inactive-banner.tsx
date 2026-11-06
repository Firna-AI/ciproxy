// The banner of an inactive organization in the Organizations tab. It
// says why the organization is inactive, when it starts its jobs, and that the
// other organizations keep running, and it offers the way on: Team for the
// plan hold, or Turn on for the hold for an Admin. On Free, it takes the place
// of the soft-limit banner of the organizations, because Team clears both. At
// a failed payment, the payment banner takes the place of this one, as it does
// of every soft-limit banner. The row of the organization says the same why.

import type { ReactNode } from "react";

import { monthly } from "../fixtures/format.ts";
import type {
  LinkedOrganization,
  OrganizationHold,
} from "../fixtures/org-links.ts";
import { organizationList, organizationsText } from "../fixtures/org.ts";
import { tiers } from "../fixtures/tiers.ts";
import { usageScenes } from "../fixtures/usage.ts";
import { Action, LinkAction } from "./actions.tsx";
import { Notice } from "./feedback.tsx";
import { paths, type CataloguePath } from "./paths.ts";
import { ChooseTeam } from "./usage-banner.tsx";

/**
 * Why a hold applies, in the words of the row and of the banner. The row
 * names the organization as "it", and the banner by its login.
 */
export function holdReason(hold: OrganizationHold, subject = "it"): string {
  const { free } = tiers;
  return hold.kind === "plan"
    ? `${free.name} includes ${organizationsText(free.organizations ?? 1)}`
    : `${hold.addedBy.name}, who added ${subject}, is no longer an Admin`;
}

/** When the organization starts its jobs, after the holds that it has. */
function startText(plan: boolean, admin: boolean): string {
  const { team } = tiers;
  if (plan && admin)
    return `It starts its jobs on ${team.name}, after an Admin turns it on.`;
  if (plan)
    return `It starts its jobs on ${team.name}, for ${monthly(team.price)}.`;
  return "It starts its jobs after an Admin turns it on.";
}

function TurnOn({ to }: { to?: CataloguePath }) {
  return to ? (
    <LinkAction to={to} tone="primary" size="sm">
      Turn on
    </LinkAction>
  ) : (
    <Action tone="primary" size="sm">
      Turn on
    </Action>
  );
}

/**
 * The banner of an organization with holds; none for an active one. Turn on
 * opens its dialog at `turnOn`.
 */
export function inactiveBanner(
  link: LinkedOrganization,
  turnOn?: CataloguePath,
): ReactNode {
  const holds = link.holds ?? [];
  if (holds.length === 0) return undefined;
  const plan = holds.some((hold) => hold.kind === "plan");
  const admin = holds.some((hold) => hold.kind === "admin");
  const machines = usageScenes["free-organizations"].machines;
  const login = link.organization.login;
  const why = holds.map((hold) => holdReason(hold, login)).join(", and ");
  return (
    <Notice
      tone="warning"
      title={`${login} is inactive`}
      actions={
        plan ? (
          <ChooseTeam
            page="other"
            usage={paths.freeOrganizations}
            plans={paths.plansInactive}
          />
        ) : (
          <TurnOn to={turnOn} />
        )
      }
    >
      {why}. {startText(plan, admin)} Jobs keep running in{" "}
      {organizationList("and")}
      {plan ? `, on all ${machines} machines` : ""}.
    </Notice>
  );
}
