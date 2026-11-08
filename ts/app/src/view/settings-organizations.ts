// The Organizations tab: each organization of the workspace with its state,
// its repository access, and the actions that its state allows. A linked
// organization belongs to the workspace, also while it is inactive or its
// App is suspended. A removed one left when an owner removed the App on
// GitHub; the tab keeps it until its data goes, with the deadline of a new
// install that keeps its job history. The API names no person and no date
// beyond that deadline, so the rows name neither.

import type { ApiFailure } from "../api/client.ts";
import type { OrganizationView } from "../api/schema.ts";
import { workspaceHref } from "../routing/pages.ts";
import type { FailureCopy } from "./claim.ts";
import { formatDeadline } from "./format.ts";

export type StatusTone = "warn" | "idle";

export type OrganizationStatus = {
  text: string;
  tone: StatusTone;
  /** The lines under the status, such as the reason of each hold. */
  lines: readonly string[];
};

export type OrganizationRow = {
  id: string;
  login: string;
  state: "linked" | "removed";
  status: OrganizationStatus | null;
  access: string;
  /** Plan and usage, where a person who pays moves to Team. */
  chooseTeam: string | null;
  turnOn: boolean;
  repositories: boolean;
  takeOut: boolean;
  installAgain: boolean;
};

/** The reason of each hold, in the words of the row. */
export const holdReasons = {
  billing_upgrade: "Free includes 1 organization",
  admin_approval: "The person who added it is no longer an Admin",
} as const;

/**
 * The organizations that the rule of the last organization counts: each
 * linked one, also an inactive or a suspended one. A removed one left the
 * workspace, so it does not count.
 */
export function linkedCount(
  organizations: readonly OrganizationView[],
): number {
  return organizations.filter((item) => item.state !== "removed").length;
}

export function accessText(organization: OrganizationView): string {
  if (organization.state === "removed") return "No repository access";
  const { access } = organization;
  if (access.mode === "all_private") return "All private repositories";
  const count = access.ids.length;
  return `${count} selected ${count === 1 ? "repository" : "repositories"}`;
}

function status(
  organization: OrganizationView,
  timeZone?: string,
): OrganizationStatus | null {
  if (organization.state === "removed") {
    const deadline = organization.delete_after ?? null;
    return {
      text: "App removed",
      tone: "idle",
      lines:
        deadline === null
          ? []
          : [
              `Install it again before ${formatDeadline(deadline, timeZone)} to keep its job history`,
            ],
    };
  }
  const { activation } = organization;
  if (activation.reasons.length > 0) {
    return {
      text: "Inactive",
      tone: "warn",
      lines: activation.reasons.map((reason) => holdReasons[reason]),
    };
  }
  if (organization.state === "suspended") {
    return { text: "App suspended", tone: "warn", lines: [] };
  }
  if (activation.preparing) {
    return {
      text: "Preparing",
      tone: "idle",
      lines: ["Its jobs start once ciproxy finishes the setup on GitHub"],
    };
  }
  return null;
}

export function organizationRows(
  organizations: readonly OrganizationView[],
  timeZone?: string,
): OrganizationRow[] {
  const severalLinked = linkedCount(organizations) > 1;
  return organizations.map((organization) => {
    const removed = organization.state === "removed";
    const { activation } = organization;
    const upgrade = activation.upgrade ?? null;
    return {
      id: organization.id,
      login: organization.login,
      state: removed ? "removed" : "linked",
      status: status(organization, timeZone),
      access: accessText(organization),
      chooseTeam:
        !removed && upgrade !== null
          ? workspaceHref("billing", upgrade.workspace_id)
          : null,
      turnOn: !removed && activation.can_enable,
      repositories: !removed,
      takeOut: !removed && severalLinked,
      installAgain: removed,
    };
  });
}

/**
 * Who added an organization that waits for an Admin. The API names the
 * person by GitHub user ID only.
 */
export function addedByText(organization: OrganizationView): string {
  const addedBy = organization.activation.added_by ?? null;
  return addedBy === null
    ? `The person who added ${organization.login} is no longer an Admin.`
    : `GitHub user ${addedBy} added ${organization.login} and is no longer an Admin.`;
}

export type OrganizationAction =
  "rename" | "take-out" | "turn-on" | "repositories";

const failureTitles: Readonly<Record<OrganizationAction, string>> = {
  rename: "Unable to rename the workspace",
  "take-out": "Unable to take the organization out",
  "turn-on": "Unable to turn on the organization",
  repositories: "Unable to save the repository access",
};

export function organizationFailure(
  failure: ApiFailure,
  action: OrganizationAction,
): FailureCopy {
  const conflict =
    failure.kind === "problem" && failure.problem.code === "conflict";
  return {
    title: failureTitles[action],
    body:
      failure.kind === "network"
        ? "Check the connection, then try again."
        : action === "take-out" && conflict
          ? "The last organization of a workspace stays in it."
          : "ciproxy could not finish the change. Try again.",
  };
}
