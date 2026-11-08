// An inactive organization in its row of the Organizations tab: the word
// Inactive with its dot and the reason of each hold under its name, and the
// action that clears a hold above the other actions of the row. The plan
// hold offers Choose Team. The hold for an Admin offers Turn on once it is
// the only hold, so with both holds the row offers the move to Team first.
// An inactive organization gets no runners, and it keeps Take out under the
// rule of the last organization.

import { Action, LinkAction } from "../../components/actions.tsx";
import { holdReason } from "../../components/inactive-banner.tsx";
import type { CataloguePath } from "../../components/paths.ts";
import { DotLabel } from "../../components/status.tsx";
import type { OrganizationHold } from "../../fixtures/org-links.ts";
import { tiers } from "../../fixtures/tiers.ts";
import { inactivePages } from "../billing/billing-scenes.tsx";

export function InactiveStatus({
  holds,
}: {
  holds: readonly OrganizationHold[];
}) {
  return (
    <span className="org-status">
      <DotLabel tone="warn">Inactive</DotLabel>
      {holds.map((hold) => (
        <span key={hold.kind} className="caption">
          {holdReason(hold)}
        </span>
      ))}
    </span>
  );
}

export function HoldActions({
  holds,
  turnOn,
}: {
  holds: readonly OrganizationHold[];
  /** The Turn on dialog, when the screen of it exists. */
  turnOn?: CataloguePath;
}) {
  const plan = holds.some((hold) => hold.kind === "plan");
  const admin = holds.some((hold) => hold.kind === "admin");
  return (
    <span className="row org-actions">
      {plan ? (
        <LinkAction to={inactivePages.plans} size="sm">
          {`Choose ${tiers.team.name}`}
        </LinkAction>
      ) : null}
      {admin && !plan && turnOn ? (
        <LinkAction to={turnOn} size="sm">
          Turn on
        </LinkAction>
      ) : null}
      {admin && !plan && !turnOn ? <Action size="sm">Turn on</Action> : null}
    </span>
  );
}
