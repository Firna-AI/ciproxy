// The parts of the return page beside the sections of the links: the
// reasons and the actions of an inactive organization, the requests that
// wait for an owner, and what failed on a link. The reasons come from the
// activation of the API alone.

import { failureStatus, type ApiFailure } from "../api/client.ts";
import type { ActivationView, SessionView } from "../api/schema.ts";
import { hasRole } from "../routing/landing.ts";
import { workspaceHomeHref, workspaceHref } from "../routing/pages.ts";
import type { LinkSection } from "./add-organization.ts";
import { joinWords } from "./format.ts";
import type { NoticeModel, PageAction } from "./page-model.ts";

/** The reasons of an inactive organization, from its activation alone. */
export function inactiveReasons(activation: ActivationView): string[] {
  return activation.reasons.map((reason) =>
    reason === "billing_upgrade"
      ? "Free includes 1 organization. It starts its jobs on Team."
      : "The person who added it is no longer an Admin. It starts its jobs after an Admin turns it on.",
  );
}

export function inactiveActions(
  activation: ActivationView,
  workspaceId: string,
): PageAction[] {
  const actions: PageAction[] = [];
  if (activation.upgrade) {
    const target = activation.upgrade.workspace_id;
    actions.push({
      kind: "go",
      label: "Plan and usage",
      href: workspaceHref("billing", target),
      tone: "primary",
    });
  }
  if (activation.can_enable) {
    actions.push({
      kind: "go",
      label: "Turn on in Settings",
      href: workspaceHref("settings", workspaceId),
      tone: "primary",
    });
  }
  return actions;
}

/**
 * The way back to a workspace of the person, for a page that waits for
 * someone else. It prefers a workspace where the person has a role.
 */
export function backAction(session: SessionView): PageAction | null {
  const home = session.workspaces.find(hasRole) ?? session.workspaces[0];
  if (home === undefined) return null;
  return {
    kind: "go",
    label: `Back to ${home.name}`,
    href: workspaceHomeHref(home.id),
    tone: "secondary",
  };
}

/** The section of install requests that wait for an owner on GitHub. */
export function requestSection(session: SessionView): LinkSection | null {
  const names = session.pending_install_requests.map(
    (item) => item.organization,
  );
  if (names.length === 0) return null;
  const list = joinWords(names, "or");
  const back = backAction(session);
  return {
    claimId: "requests",
    title: `Waiting for an owner of ${list}`,
    lead: `GitHub sent your request to the owners of ${list}. When one of them approves it, sign in to ciproxy again to choose its workspace.`,
    notice: null,
    choices: null,
    submit: null,
    waiting: null,
    reasons: [],
    actions: [
      { kind: "check-again", label: "Check again", tone: "primary" },
      ...(back === null ? [] : [back]),
    ],
  };
}

/** What failed on a link, and what to do next. */
export function linkFailure(failure: ApiFailure): NoticeModel {
  const status = failureStatus(failure);
  const known: Record<number, NoticeModel> = {
    403: {
      tone: "danger",
      title: "Unable to confirm the install on GitHub",
      body: "Check again, or sign in with GitHub again.",
    },
    409: {
      tone: "danger",
      title: "The organization changed meanwhile",
      body: "Its current state shows here. Check again to read it once more.",
    },
    503: {
      tone: "danger",
      title: "GitHub did not answer",
      body: "Try again in a moment.",
    },
  };
  if (status === null) {
    return {
      tone: "danger",
      title: "Unable to reach ciproxy",
      body: "Check the connection, then try again.",
    };
  }
  return (
    known[status] ?? {
      tone: "danger",
      title: "Unable to finish the add",
      body: "Try again.",
    }
  );
}
