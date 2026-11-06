// The frame of a workspace page: the navigation for the role, the workspace
// switch, the account menu, and the banner of the organization soft limit.
// Settings and Add an organization are for Admins, on every tier.

import type {
  SessionView,
  WorkspaceRole,
  WorkspaceView,
} from "../api/schema.ts";
import {
  workspaceHomeHref,
  workspaceHref,
  type WorkspacePage,
} from "../routing/pages.ts";
import { roleNames } from "./format.ts";

export type NavItem = {
  key: WorkspacePage;
  label: string;
  mobileLabel: string;
  href: string;
  /** The progress of Setup, such as "1 of 3". */
  count: string | null;
};

export const pageTitles: Readonly<Record<WorkspacePage, string>> = {
  setup: "Set up",
  jobs: "Jobs",
  machines: "Machines",
  savings: "Savings",
  billing: "Plan and usage",
  settings: "Settings",
};

const mainPages: readonly WorkspacePage[] = [
  "jobs",
  "machines",
  "savings",
  "billing",
  "settings",
];

const mobileLabels: Readonly<Record<WorkspacePage, string>> = {
  ...pageTitles,
  billing: "Plan",
};

export type SetupProgress = { done: number; total: number; complete: boolean };

/** The pages that the role sees, with Setup first while it is open. */
export function navItems(
  workspaceId: string,
  role: WorkspaceRole,
  setup: SetupProgress | null,
): NavItem[] {
  const item = (key: WorkspacePage, count: string | null = null): NavItem => ({
    key,
    label: pageTitles[key],
    mobileLabel: mobileLabels[key],
    href: workspaceHref(key, workspaceId),
    count,
  });
  const items = mainPages
    .filter((key) => key !== "settings" || role === "admin")
    .map((key) => item(key));
  if (role === "admin" && setup !== null && !setup.complete) {
    items.unshift(item("setup", `${setup.done} of ${setup.total}`));
  }
  return items;
}

export type WorkspaceChoiceItem = {
  id: string;
  name: string;
  detail: string;
  current: boolean;
  href: string;
};

export type WorkspaceMenu = {
  choices: readonly WorkspaceChoiceItem[];
  /** Add an organization to the current workspace: Admins only. */
  canAddOrganization: boolean;
  /** The install on another organization, which the API names. */
  installUrl: string;
};

export function workspaceMenu(
  session: SessionView,
  current: WorkspaceView,
): WorkspaceMenu {
  return {
    choices: session.workspaces.map((workspace) => ({
      id: workspace.id,
      name: workspace.name,
      detail: workspace.role ? roleNames[workspace.role] : "No role yet",
      current: workspace.id === current.id,
      href: workspaceHomeHref(workspace.id),
    })),
    canAddOrganization: current.role === "admin",
    installUrl: session.install_url,
  };
}

export type AccountMenu = {
  login: string;
  roleName: string;
  workspaceName: string;
  /** An owner with the Billing or the Viewer role claims Admin here. */
  canClaim: boolean;
};

export function accountMenu(
  session: SessionView,
  workspace: WorkspaceView,
  role: WorkspaceRole,
): AccountMenu {
  return {
    login: session.login,
    roleName: roleNames[role],
    workspaceName: workspace.name,
    canClaim: workspace.can_claim_admin && role !== "admin",
  };
}

export type Banner = {
  title: string;
  body: string;
  /** A link to Plan and usage for a person who pays, or who to ask. */
  action: { label: string; href: string } | { ask: string };
};

/** The banner of every page while the workspace passes the Free limit. */
export function organizationBanner(
  workspace: WorkspaceView,
  role: WorkspaceRole,
): Banner | null {
  if (!workspace.organization_limit_exceeded) return null;
  return {
    title: "Free includes 1 organization",
    body: "Jobs keep running in every active organization of this workspace. Team runs several organizations on the same machines.",
    action:
      role === "viewer"
        ? { ask: "Ask an Admin or a Billing person" }
        : {
            label: "Plan and usage",
            href: workspaceHref("billing", workspace.id),
          },
  };
}
