// Where the dashboard opens. The session decides it: a removed App, an
// ended access, a personal-account install, an install that waits for the
// person, or a workspace. The client never infers a workspace for an
// organization; it only opens the page that the server state asks for.

import type {
  OrganizationLinkView,
  SessionView,
  SetupView,
  WorkspaceView,
} from "../api/schema.ts";
import { isOpenLink } from "../state/links.ts";
import { setupWaits } from "../state/setup.ts";
import { pages, workspaceHref } from "./pages.ts";

export type Landing = { href: string } | { workspace: WorkspaceView };

export type LandingContext = {
  /** This tab followed an install link to GitHub and came back. */
  installStarted: boolean;
  /** The workspace that the URL or the last visit names. */
  preferredWorkspaceId: string | null;
};

export function openLinks(session: SessionView): OrganizationLinkView[] {
  return session.organization_links.filter(isOpenLink);
}

/** A role that the API omitted is no role, as null is. */
export function hasRole(workspace: WorkspaceView): boolean {
  return workspace.role !== null && workspace.role !== undefined;
}

/** The workspace to open: the preferred one, else the first with a role. */
export function pickWorkspace(
  workspaces: readonly WorkspaceView[],
  preferredId: string | null,
): WorkspaceView | null {
  return (
    workspaces.find((workspace) => workspace.id === preferredId) ??
    workspaces.find((workspace) => hasRole(workspace)) ??
    workspaces[0] ??
    null
  );
}

/**
 * A setup outcome stays in the sign-in. A person without a workspace has
 * nothing else to see, so it always counts. A person with a workspace sees
 * it only on the return of an install that this tab started.
 */
function freshOutcome(session: SessionView, context: LandingContext): boolean {
  return session.state !== "ready" || context.installStarted;
}

export function sessionLanding(
  session: SessionView,
  context: LandingContext,
): Landing {
  if (session.state === "app_removed") return { href: pages.appRemoved };
  if (session.state === "access_ended") return { href: pages.accessEnded };
  const fresh = freshOutcome(session, context);
  if (fresh && session.setup?.kind === "organizations_only") {
    return { href: pages.personalAccount };
  }
  if (openLinks(session).length > 0) return { href: pages.addOrganization };
  if (session.state === "install_required") return { href: pages.install };
  if (
    context.installStarted &&
    session.setup?.kind === "waiting_approval" &&
    session.pending_install_requests.length > 0
  ) {
    return { href: pages.addOrganization };
  }
  const workspace = pickWorkspace(
    session.workspaces,
    context.preferredWorkspaceId,
  );
  return workspace === null ? { href: pages.install } : { workspace };
}

/** An Admin opens Setup until it is done; everyone else opens Jobs. */
export function workspaceLanding(
  workspace: WorkspaceView,
  setup: SetupView | null,
): string {
  if (workspace.role === "admin" && setup !== null && setupWaits(setup)) {
    return workspaceHref("setup", workspace.id);
  }
  return workspaceHref("jobs", workspace.id);
}
