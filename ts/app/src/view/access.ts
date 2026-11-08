// The pages that show in place of the dashboard: sign-in, the App install,
// the install on a personal account, the removed App, the ended access, and
// the page of a member with no role. Each reads only the session, so no
// page shows data that the API did not give.

import { githubSignInHref } from "../api/endpoints.ts";
import type {
  RemovedOrganization,
  SessionView,
  WorkspaceView,
} from "../api/schema.ts";
import type { SignInReason } from "../routing/pages.ts";
import { formatDeadline, joinWords, signedInAs } from "./format.ts";
import {
  installLink,
  leave,
  type AuthPageModel,
  type NoticeModel,
} from "./page-model.ts";

const signInNotices: Readonly<Record<SignInReason, NoticeModel>> = {
  github: {
    tone: "danger",
    title: "GitHub sign-in did not finish",
    body: "Sign in with GitHub again to continue.",
  },
  ended: {
    tone: "info",
    title: "Your sign-in ended after 30 days",
    body: "Sign in with GitHub again to continue.",
  },
};

export function signInPage(reason: SignInReason | null): AuthPageModel {
  return {
    title: "Run GitHub Actions on your own machines",
    lead: "ciproxy starts each job on your machines first, and on Boat when they are busy.",
    notice: reason === null ? null : signInNotices[reason],
    list: null,
    actions: [leave("Sign in with GitHub", githubSignInHref, "primary")],
    footer: "",
  };
}

/** The permissions of the GitHub App, as the security doc states them. */
export const appPermissions: readonly string[] = [
  "Self-hosted runners · read and write, to start a runner for each job",
  "Metadata · read, to list your repositories",
  "Actions · read, to read the branch and the event of each run",
  "Members · read, see who belongs to your organization, so that ciproxy can give roles",
];

function waitingNotice(session: SessionView): NoticeModel {
  const names = session.pending_install_requests.map(
    (item) => item.organization,
  );
  if (names.length === 0) {
    return {
      tone: "info",
      title: "Waiting for an owner to approve",
      body: "GitHub sent your request to the owners of the organization. When one of them approves it, you become the first Admin in ciproxy, and setup continues here.",
    };
  }
  const list = joinWords(names, "or");
  const next =
    names.length === 1
      ? `When one of them approves it, you become the first Admin of ${list} in ciproxy, and setup continues here.`
      : "When an owner approves one, you become its first Admin in ciproxy, and setup continues here.";
  return {
    tone: "info",
    title: `Waiting for an owner of ${list}`,
    body: `GitHub sent your request to the owners of ${list}. ${next}`,
  };
}

export function installPage(session: SessionView): AuthPageModel {
  const footer = signedInAs(session.login);
  if (session.reauthorization_required && session.reauthorization_url) {
    return {
      title: "Your workspace is ready",
      lead: "Sign in with GitHub again to open it.",
      notice: null,
      list: null,
      actions: [
        leave("Sign in with GitHub", session.reauthorization_url, "primary"),
      ],
      footer,
    };
  }
  const base = {
    title: "Install ciproxy on GitHub",
    lead: "Pick the organization and its repositories on GitHub. ciproxy then runs their jobs that use runs-on: ciproxy.",
    footer,
  };
  const waiting =
    session.setup?.kind === "waiting_approval" ||
    session.pending_install_requests.length > 0;
  if (waiting) {
    return {
      ...base,
      notice: waitingNotice(session),
      list: null,
      actions: [
        { kind: "check-again", label: "Check again", tone: "primary" },
        installLink(
          "Install on another organization",
          session.install_url,
          "secondary",
        ),
      ],
    };
  }
  return {
    ...base,
    notice: null,
    list: { caption: "The app asks for", items: appPermissions },
    actions: [installLink("Install on GitHub", session.install_url, "primary")],
  };
}

export function personalAccountPage(
  session: SessionView,
  removeUrl: string,
): AuthPageModel {
  return {
    title: "ciproxy works with GitHub organizations",
    lead: `You installed it on ${session.login}, your personal account. Install it on an organization instead. A GitHub organization is free.`,
    notice: null,
    list: null,
    actions: [
      installLink("Install on an organization", session.install_url, "primary"),
      leave(`Remove from ${session.login}`, removeUrl, "secondary", "external"),
    ],
    footer: signedInAs(session.login),
  };
}

/**
 * The removed App. The API names the organization and the last moment of a
 * new install that keeps the history; without them the page says only what
 * stays true.
 */
export function appRemovedPage(
  session: SessionView,
  removed: RemovedOrganization | null,
  timeZone?: string,
): AuthPageModel {
  const after =
    "Each machine then needs the install command again, and Boat a new connection.";
  return {
    title:
      removed === null
        ? "The GitHub App was removed"
        : `The GitHub App was removed from ${removed.login}`,
    lead:
      removed === null
        ? "Its jobs no longer start on ciproxy."
        : `Jobs of ${removed.login} no longer start on ciproxy.`,
    notice:
      removed === null
        ? null
        : {
            tone: "info",
            title: `Install it again before ${formatDeadline(removed.delete_after, timeZone)}`,
            body: `ciproxy keeps the labels, settings, and job history until then. ${after}`,
          },
    list: null,
    actions: [installLink("Install on GitHub", session.install_url, "primary")],
    footer: signedInAs(session.login),
  };
}

/**
 * The ended access. The API says only that an earlier role or request
 * ended, with no cause and no workspace, so the page names neither.
 */
export function accessEndedPage(session: SessionView): AuthPageModel {
  return {
    title: "Your access ended",
    lead: "Your role or your request for access ended. Ask an Admin of your workspace for access, then check again.",
    notice: null,
    list: null,
    actions: [
      { kind: "check-again", label: "Check again", tone: "primary" },
      installLink(
        "Install on another organization",
        session.install_url,
        "secondary",
      ),
    ],
    footer: signedInAs(session.login),
  };
}

export function noRolePage(
  session: SessionView,
  workspace: WorkspaceView,
): AuthPageModel {
  const claim = workspace.can_claim_admin;
  return {
    title: `Waiting for an Admin of ${workspace.name}`,
    lead: `ciproxy sent your request for access to the Admins. When one of them gives you a role, ${workspace.name} opens here.`,
    notice: claim
      ? {
          tone: "info",
          title: "You own an organization of this workspace on GitHub",
          body: "An owner can claim the Admin role at once.",
        }
      : null,
    list:
      workspace.admins.length > 0
        ? { caption: `Admins of ${workspace.name}`, items: workspace.admins }
        : null,
    actions: claim
      ? [
          {
            kind: "claim-admin",
            label: "Claim the Admin role",
            tone: "primary",
            workspaceId: workspace.id,
          },
          { kind: "check-again", label: "Check again", tone: "secondary" },
        ]
      : [
          { kind: "check-again", label: "Check again", tone: "primary" },
          installLink(
            "Install on another organization",
            session.install_url,
            "secondary",
          ),
        ],
    footer: signedInAs(session.login),
  };
}

/**
 * What Check again does. A sign-in with no workspace session needs fresh
 * GitHub proof, through the URL that the API gives. Any other sign-in reads
 * the server state again.
 */
export function checkAgainTarget(
  session: SessionView,
): { leave: string } | { refresh: true } {
  if (session.state !== "ready" && session.reauthorization_url) {
    return { leave: session.reauthorization_url };
  }
  return { refresh: true };
}
