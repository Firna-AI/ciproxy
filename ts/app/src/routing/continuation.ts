// The continuation of the last verified install. GitHub sends every install
// return to the landing, and the session keeps the verified organization of
// the install for the whole sign-in. The landing acts on that install once:
// when its installation is new to this browser, its claim names where the
// person goes next. The claim of the same installation ID decides it; no
// remembered workspace replaces it. Only an ended continuation of the
// latest sign-in, which names its exact claims, opens before its workspace.
//
// A browser keeps the last installation that the landing handled, and that
// installation is never new again, also when a later install was cancelled.
// For any other installation, a sign-in with a workspace session counts it as
// new, because its claims show their true state. A sign-in without a
// workspace session counts it as new only when this tab started an install:
// its claims show old completed installs as open, and the removed-App and
// ended-access pages must stay for them, whatever the browser stored.

import type { SessionView, WorkspaceView } from "../api/schema.ts";
import { isOpenLink } from "../state/links.ts";

/** The stored value when the landing saw no verified organization install. */
export const noVerifiedInstallation = "";

export type InstallMemory = {
  /** This tab followed an install link to GitHub. */
  installStarted: boolean;
  /** The stored last handled installation, or null when nothing is stored. */
  handledInstallationId: string | null;
};

export type InstallContinuation =
  /** No verified install that is new to this browser. */
  | { kind: "none" }
  /** A new install that no claim of the person names yet. */
  | { kind: "unresolved"; installationId: string }
  /** A new install whose claim still waits for the person or GitHub. */
  | { kind: "return"; installationId: string }
  /** A new install whose claim joined a workspace that the person can open. */
  | { kind: "workspace"; installationId: string; workspace: WorkspaceView }
  /** A new install whose claim ended in a workspace outside the session. */
  | { kind: "settled"; installationId: string };

/** The installation that the session verified for an organization, if any. */
export function verifiedInstallation(session: SessionView): string | null {
  return session.setup?.kind === "organization"
    ? session.setup.installation_id
    : null;
}

/** Whether the verified installation is new to this browser. */
function isNew(
  session: SessionView,
  installationId: string,
  memory: InstallMemory,
): boolean {
  if (memory.handledInstallationId === installationId) return false;
  return session.state === "ready" || memory.installStarted;
}

export function installContinuation(
  session: SessionView,
  memory: InstallMemory,
): InstallContinuation {
  const installationId = verifiedInstallation(session);
  if (installationId === null || !isNew(session, installationId, memory)) {
    return { kind: "none" };
  }
  const claim = session.organization_links.find(
    (link) => link.installation_id === installationId,
  );
  if (claim === undefined) return { kind: "unresolved", installationId };
  if (isOpenLink(claim)) return { kind: "return", installationId };
  const workspace = session.workspaces.find(
    (item) => item.id === claim.workspace_id,
  );
  return workspace === undefined
    ? { kind: "settled", installationId }
    : { kind: "workspace", installationId, workspace };
}

/**
 * Whether the install stays open after the landing: its claim still waits,
 * or no claim names it yet. Then the browser keeps it new, so the next
 * landing, after fresh GitHub proof or a late completion, opens the same
 * claim's workspace.
 */
export function installStaysOpen(continuation: InstallContinuation): boolean {
  return continuation.kind === "return" || continuation.kind === "unresolved";
}

/**
 * The value that the browser stores after the landing used the session, or
 * null to keep the stored value. A new install counts as handled only when
 * the landing reached its final destination. Without a stored value, the
 * landing stores the current install, or the empty value, without acting on
 * an install that is not new.
 */
export function installationToRemember(
  session: SessionView,
  memory: InstallMemory,
  continuation: InstallContinuation,
): string | null {
  switch (continuation.kind) {
    case "workspace":
    case "settled":
      return continuation.installationId;
    case "return":
    case "unresolved":
      return null;
    case "none":
      return memory.handledInstallationId === null
        ? (verifiedInstallation(session) ?? noVerifiedInstallation)
        : null;
  }
}
