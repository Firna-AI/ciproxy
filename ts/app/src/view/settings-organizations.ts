// The Organizations tab: each organization of the workspace with the day it
// joined, its state, its repository access with the counts of the latest
// complete repository snapshot, and the actions that its state allows. A
// linked organization belongs to the workspace, also while it is inactive or
// its App is suspended. A removed one left when an owner removed the App on
// GitHub; the tab keeps it until its data goes, with the deadline of a new
// install that keeps its job history. A day or a count that the API does not
// know stays out.

import type { ApiFailure } from "../api/client.ts";
import type { OrganizationView } from "../api/schema.ts";
import { workspaceHref } from "../routing/pages.ts";
import type { FailureCopy } from "./claim.ts";
import { formatDay, formatDeadline } from "./format.ts";

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
  /** "Since 1 September": the day it joined, for a linked organization. */
  since: string | null;
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

/**
 * The person who added an organization: the name that ciproxy kept, else the
 * login, else the GitHub ID; null when the API names nobody.
 */
export function adderName(organization: OrganizationView): string | null {
  const person = organization.added_by ?? null;
  if (person !== null) {
    return person.name ?? person.login ?? `GitHub user ${person.id}`;
  }
  const id = organization.activation.added_by ?? null;
  return id === null ? null : `GitHub user ${id}`;
}

/**
 * The reason of a hold, in the words of a row or a banner. `subject` names
 * the organization, and `lead` says whether the reason starts a sentence.
 */
export function holdReason(
  organization: OrganizationView,
  reason: OrganizationView["activation"]["reasons"][number],
  subject = "it",
  lead = true,
): string {
  if (reason === "billing_upgrade") return "Free includes 1 organization";
  const adder = adderName(organization);
  if (adder !== null)
    return `${adder}, who added ${subject}, is no longer an Admin`;
  return `${lead ? "The" : "the"} person who added ${subject} is no longer an Admin`;
}

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

function repositoriesWord(count: string): string {
  return count === "1" ? "repository" : "repositories";
}

/**
 * The repository access of an organization, with the counts of its latest
 * complete repository snapshot: "All 14 private repositories" or "3 of 5
 * private repositories". Without a snapshot it names the policy only.
 */
export function accessText(organization: OrganizationView): string {
  if (organization.state === "removed") return "No repository access";
  const all = organization.access.mode === "all_private";
  const counts = organization.repositories ?? null;
  if (counts === null) {
    return all ? "All private repositories" : "Selected private repositories";
  }
  const total = counts.private;
  if (total === "0") return "No private repositories";
  const word = repositoriesWord(total);
  return all
    ? `All ${total} private ${word}`
    : `${counts.allowed_private} of ${total} private ${word}`;
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
      lines: activation.reasons.map((reason) =>
        holdReason(organization, reason),
      ),
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
    const joined = organization.joined_at ?? null;
    return {
      id: organization.id,
      login: organization.login,
      since:
        removed || joined === null
          ? null
          : `Since ${formatDay(joined, timeZone)}`,
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
 * Who added an organization that waits for an Admin, and on which day:
 * "Ana Souza added mokly-studio on 8 December and is no longer an Admin."
 */
export function addedByText(
  organization: OrganizationView,
  timeZone?: string,
): string {
  const { login } = organization;
  const adder = adderName(organization);
  if (adder === null) {
    return `The person who added ${login} is no longer an Admin.`;
  }
  const joined = organization.joined_at ?? null;
  const day = joined === null ? "" : ` on ${formatDay(joined, timeZone)}`;
  return `${adder} added ${login}${day} and is no longer an Admin.`;
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
