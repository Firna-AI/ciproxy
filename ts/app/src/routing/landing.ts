// Where the dashboard opens. The session decides it: a role that waits for a
// new GitHub sign-in, the continuation of a new install, a removed App, an
// ended access, a personal-account install, the exact claims of the latest
// sign-in, an install that waits for the person, or a workspace. The client
// never infers a workspace for an organization; it only opens the page that
// the server state asks for.

import type {
  OrganizationLinkView,
  SessionView,
  SetupView,
  WorkspaceView,
} from "../api/schema.ts";
import { isOpenLink } from "../state/links.ts";
import { setupWaits } from "../state/setup.ts";
import {
  claimContinuation,
  continuationToRemember,
  type ClaimContinuation,
} from "./claim-continuation.ts";
import {
  installContinuation,
  installationToRemember,
  installStaysOpen,
  type InstallContinuation,
} from "./continuation.ts";
import { pages, workspaceHref } from "./pages.ts";

export type Landing = { href: string } | { workspace: WorkspaceView };

export type LandingContext = {
  /** This tab followed an install link to GitHub and came back. */
  installStarted: boolean;
  /** The workspace that the URL or the last visit names. */
  preferredWorkspaceId: string | null;
  /** The stored last handled installation; absent or null when none. */
  handledInstallationId?: string | null;
  /** The stored key of the last handled ended continuation, if any. */
  handledContinuation?: string | null;
};

/** The page to open, and what the browser keeps about the install. */
export type LandingDecision = {
  landing: Landing;
  /** The installation to store once the page opens, or null for no change. */
  remember: string | null;
  /** The install stays open, so the tab keeps its install mark. */
  installOpen: boolean;
  /** The ended continuation to store once the page opens, or null. */
  rememberContinuation: string | null;
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

/**
 * The pages that come before the destination of an ended continuation. A
 * sign-in without a workspace session whose role or request is ready signs
 * in again first, as the API asks. The claim of a new install comes next,
 * also before the removed-App and ended-access pages.
 */
function firstLanding(
  session: SessionView,
  context: LandingContext,
  install: InstallContinuation,
): Landing | null {
  if (
    session.state !== "ready" &&
    session.reauthorization_required &&
    session.reauthorization_url
  ) {
    return { href: pages.install };
  }
  if (install.kind === "return") return { href: pages.addOrganization };
  if (session.state === "app_removed") return { href: pages.appRemoved };
  if (session.state === "access_ended") return { href: pages.accessEnded };
  if (
    freshOutcome(session, context) &&
    session.setup?.kind === "organizations_only"
  ) {
    return { href: pages.personalAccount };
  }
  return null;
}

/**
 * The rest of the landing. The exact claims of the latest sign-in come
 * first: one ended claim opens its workspace, before any remembered one,
 * and several show their outcomes on the return page. The workspace of a
 * new install follows. Older claims keep their place after it.
 */
function laterLanding(
  session: SessionView,
  context: LandingContext,
  install: InstallContinuation,
  claims: ClaimContinuation,
): Landing {
  if (claims.kind === "outcomes") return { href: pages.addOrganization };
  if (claims.kind === "workspace") return { workspace: claims.workspace };
  if (install.kind === "workspace") return { workspace: install.workspace };
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

export function landingDecision(
  session: SessionView,
  context: LandingContext,
): LandingDecision {
  const memory = {
    installStarted: context.installStarted,
    handledInstallationId: context.handledInstallationId ?? null,
  };
  const install = installContinuation(session, memory);
  const claims = claimContinuation(
    session,
    context.handledContinuation ?? null,
  );
  const first = firstLanding(session, context, install);
  return {
    landing: first ?? laterLanding(session, context, install, claims),
    remember: installationToRemember(session, memory, install),
    installOpen: installStaysOpen(install),
    // An earlier page leaves the continuation for a later landing.
    rememberContinuation:
      first === null ? continuationToRemember(claims) : null,
  };
}

export function sessionLanding(
  session: SessionView,
  context: LandingContext,
): Landing {
  return landingDecision(session, context).landing;
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
