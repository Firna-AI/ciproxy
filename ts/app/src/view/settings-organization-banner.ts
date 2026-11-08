// The banner of the Organizations tab while an organization is inactive. It
// names the organization and its reasons, says when it starts its jobs, and
// says that the other organizations keep running, as "Inactive
// organizations" in docs/protocol/workspaces.md says. It takes the place of
// the soft-limit banner of the shell on that tab.

import type { OrganizationView } from "../api/schema.ts";
import { workspaceHref } from "../routing/pages.ts";
import { holdReasons } from "./settings-organizations.ts";

/** An organization that runs its jobs now: active, admitted, with no hold. */
function running(organization: OrganizationView): boolean {
  const { activation } = organization;
  return (
    organization.state === "active" &&
    activation.reasons.length === 0 &&
    !activation.preparing
  );
}

export type InactiveBanner = {
  title: string;
  body: string;
  /** Plan and usage, when a hold waits for Team. */
  chooseTeam: string | null;
  /** The one organization that Turn on clears, when it is the only hold. */
  turnOn: { id: string; login: string } | null;
};

function joinLogins(logins: readonly string[]): string {
  if (logins.length <= 1) return logins[0] ?? "";
  return `${logins.slice(0, -1).join(", ")} and ${logins[logins.length - 1]}`;
}

/**
 * The banner of the tab while an organization is inactive. It names the
 * organization and its reasons, and says that the other organizations keep
 * running, as "Inactive organizations" in docs/protocol/workspaces.md says.
 */
export function inactiveBanner(
  organizations: readonly OrganizationView[],
): InactiveBanner | null {
  const held = organizations.filter(
    (item) => item.state !== "removed" && item.activation.reasons.length > 0,
  );
  const first = held[0];
  if (first === undefined) return null;
  const others = organizations.filter(running).map((item) => item.login);
  const keep =
    others.length > 0 ? ` Jobs keep running in ${joinLogins(others)}.` : "";
  const upgrade =
    held.find((item) => item.activation.upgrade)?.activation.upgrade ?? null;
  const chooseTeam =
    upgrade === null ? null : workspaceHref("billing", upgrade.workspace_id);
  if (held.length > 1) {
    return {
      title: `${held.length} organizations are inactive`,
      body: `Each row says why.${keep}`,
      chooseTeam,
      turnOn: null,
    };
  }
  const reasons = first.activation.reasons;
  const plan = reasons.includes("billing_upgrade");
  const admin = reasons.includes("admin_approval");
  const why = reasons
    .map((reason) =>
      reason === "admin_approval"
        ? `the person who added ${first.login} is no longer an Admin`
        : holdReasons[reason],
    )
    .join(", and ");
  const start =
    plan && admin
      ? "It starts its jobs on Team, after an Admin turns it on."
      : plan
        ? "It starts its jobs on Team."
        : "It starts its jobs after an Admin turns it on.";
  return {
    title: `${first.login} is inactive`,
    body: `${why.charAt(0).toUpperCase()}${why.slice(1)}. ${start}${keep}`,
    chooseTeam,
    turnOn: first.activation.can_enable
      ? { id: first.id, login: first.login }
      : null,
  };
}
