// The return page of an install on GitHub: one section for each link of the
// person, and one for install requests that wait for an owner. GitHub picks
// the organization; the person picks the workspace here, from the offers of
// the API, with none picked at first. The page never offers a new workspace.

import type { OrganizationLinkView, SessionView } from "../api/schema.ts";
import { workspaceHomeHref } from "../routing/pages.ts";
import type { LinkEntry } from "../state/links.ts";
import { backAction, inactiveActions, inactiveReasons } from "./link-notes.ts";
import { leave, type NoticeModel, type PageAction } from "./page-model.ts";

export type WorkspaceChoiceModel = {
  id: string;
  name: string;
  /** Why the organization would join this workspace inactive, if it would. */
  note: string | null;
  checked: boolean;
};

export type LinkSection = {
  claimId: string;
  title: string;
  lead: string | null;
  notice: NoticeModel | null;
  /** The choice of the workspace, when the link waits for it. */
  choices: readonly WorkspaceChoiceModel[] | null;
  /** The add button of the choice: its label, and whether it can act. */
  submit: { label: string; enabled: boolean; busy: boolean } | null;
  /** A line with a spinner, while GitHub or ciproxy works. */
  waiting: string | null;
  /** The reasons why a linked organization stays inactive. */
  reasons: readonly string[];
  actions: readonly PageAction[];
};

function workspaceName(
  session: SessionView,
  link: OrganizationLinkView,
): string {
  const id = link.workspace_id;
  return (
    session.workspaces.find((item) => item.id === id)?.name ??
    link.workspaces.find((item) => item.id === id)?.name ??
    "your workspace"
  );
}

function chooseSection(
  entry: LinkEntry,
): Pick<LinkSection, "title" | "lead" | "choices" | "submit"> {
  const { link, picked } = entry;
  const chosen = link.workspaces.find((choice) => choice.id === picked);
  return {
    title: `Choose a workspace for ${link.organization}`,
    lead: `GitHub installed ciproxy on ${link.organization}. Pick the workspace that runs its jobs.`,
    choices: link.workspaces.map((choice) => ({
      id: choice.id,
      name: choice.name,
      note: choice.requires_upgrade
        ? `${link.organization} stays inactive here until the workspace moves to Team.`
        : null,
      checked: choice.id === picked,
    })),
    submit: {
      label: chosen
        ? `Add ${link.organization} to ${chosen.name}`
        : `Add ${link.organization}`,
      enabled: chosen !== undefined && !entry.choosing,
      busy: entry.choosing,
    },
  };
}

/** The actions of a waiting page, with the way back to a workspace. */
function withBack(session: SessionView, actions: PageAction[]): PageAction[] {
  const back = backAction(session);
  return back === null ? actions : [...actions, back];
}

export function linkSection(
  session: SessionView,
  entry: LinkEntry,
): LinkSection {
  const { link } = entry;
  const org = link.organization;
  const section: LinkSection = {
    claimId: link.id,
    title: "",
    lead: null,
    notice: null,
    choices: null,
    submit: null,
    waiting: null,
    reasons: [],
    actions: [],
  };
  const checkAgain: PageAction = {
    kind: "check-again",
    label: "Check again",
    tone: "primary",
  };
  switch (link.state) {
    case "choose_workspace":
      return { ...section, ...chooseSection(entry) };
    case "reauthorization_required":
      return {
        ...section,
        title:
          session.state === "ready"
            ? "Add an organization"
            : "Install ciproxy on GitHub",
        notice: {
          tone: "danger",
          title: "Unable to confirm the install on GitHub",
          body: "Sign in with GitHub again, so that ciproxy can check the install with your account.",
        },
        actions: withBack(session, [
          leave("Sign in with GitHub", link.reauthorization_url, "primary"),
        ]),
      };
    case "waiting_for_admin":
      return {
        ...section,
        title: `${org} waits for a workspace`,
        lead: `GitHub installed ciproxy on ${org}. To add it, ask an Admin of your workspace for the Admin role, then check again.`,
        actions: withBack(session, [checkAgain]),
      };
    case "confirming_installation":
      return {
        ...section,
        title: `Confirming the install on ${org}`,
        lead: `GitHub installed ciproxy on ${org}. This page updates when GitHub confirms it.`,
        waiting: "Waiting for GitHub…",
        actions: [{ ...checkAgain, tone: "secondary" }],
      };
    case "linking":
      return {
        ...section,
        title: `Adding ${org} to ${workspaceName(session, link)}`,
        lead: "This page updates when the organization joins.",
        waiting: "Adding the organization…",
      };
    case "linked":
    case "linked_inactive":
      return linkedSection(session, entry, section);
  }
}

function linkedSection(
  session: SessionView,
  entry: LinkEntry,
  section: LinkSection,
): LinkSection {
  const { link } = entry;
  const name = workspaceName(session, link);
  const workspaceId = link.workspace_id;
  const open: PageAction[] =
    workspaceId === null || workspaceId === undefined
      ? []
      : [
          {
            kind: "go",
            label: `Open ${name}`,
            href: workspaceHomeHref(workspaceId),
            tone: "secondary",
          },
        ];
  const activation = link.activation;
  if (
    link.state === "linked" ||
    activation === null ||
    activation === undefined
  ) {
    return {
      ...section,
      title: `${link.organization} joined ${name}`,
      lead: `Its jobs use the machines, labels, and settings of ${name} now.`,
      actions: open.map((action) => ({ ...action, tone: "primary" })),
    };
  }
  const preparing = activation.preparing && activation.reasons.length === 0;
  return {
    ...section,
    title: `${link.organization} joined ${name}, inactive`,
    lead: preparing
      ? null
      : `Jobs keep running in the other organizations of ${name}.`,
    waiting: preparing ? "Preparing its runners…" : null,
    reasons: inactiveReasons(activation),
    notice: activation.ask_admin
      ? {
          tone: "info",
          title: "Ask an Admin",
          body: "An Admin or a Billing person of the workspace can end these holds.",
        }
      : null,
    actions: [...inactiveActions(activation, workspaceId ?? ""), ...open],
  };
}
