// The exact continuation of the latest GitHub sign-in. The session names the
// claims that this sign-in proved or completed, for example an install that
// an owner approved after its return. Only these names count: no claim
// order, time, role, or remembered workspace picks a claim.
//
// The landing opens the destination of an ended continuation once: the
// workspace of one ended claim, or the return page with the outcomes of
// several. A named claim that still waits adds no route of its own, so the
// removed-App and ended-access pages keep their place, and the open links of
// the session lead to the return page as before. A browser-only session
// masks some claims as `reauthorization_required`, also completed history.
// It does not mask a genuine `waiting_for_admin` claim that way. So a
// `reauthorization_required` view names no continuation.
//
// The browser stores the key of an ended continuation once the landing opens
// its destination, or once the person leaves the return page. A stored key
// names only ended claims, so it never hides a claim that still waits.

import type {
  OrganizationLinkView,
  SessionView,
  WorkspaceView,
} from "../api/schema.ts";
import { isOpenLink } from "../state/links.ts";

export type ClaimContinuation =
  /** No named claim, or this browser handled the ended continuation. */
  | { kind: "none" }
  /** A named claim still waits for the person, GitHub, or ciproxy. */
  | { kind: "waiting" }
  /** Several named claims ended; the return page shows each outcome. */
  | { kind: "outcomes"; key: string }
  /** One named claim ended in a workspace that the person can open. */
  | { kind: "workspace"; key: string; workspace: WorkspaceView }
  /** One named claim ended in a workspace outside the session. */
  | { kind: "settled"; key: string };

/**
 * The links that the continuation names, each in its newest known state. A
 * `reauthorization_required` view names no continuation: the sign-in just
 * proved its claim, so the view masks the real state.
 */
export function namedLinks(
  session: SessionView,
  newer: readonly OrganizationLinkView[] = [],
): OrganizationLinkView[] {
  const named = new Set(session.installation_continuation);
  return session.organization_links
    .filter((link) => named.has(link.id))
    .map((link) => newer.find((item) => item.id === link.id) ?? link)
    .filter((link) => link.state !== "reauthorization_required");
}

/** The claim IDs in a fixed order, so that the order of the API adds nothing. */
function keyOf(links: readonly OrganizationLinkView[]): string {
  return links
    .map((link) => link.id)
    .sort()
    .join(",");
}

/** The key of the continuation once every named claim ended, else null. */
export function endedContinuationKey(
  session: SessionView,
  newer: readonly OrganizationLinkView[] = [],
): string | null {
  const links = namedLinks(session, newer);
  return links.length === 0 || links.some(isOpenLink) ? null : keyOf(links);
}

export function claimContinuation(
  session: SessionView,
  handled: string | null,
): ClaimContinuation {
  const links = namedLinks(session);
  const [first, ...others] = links;
  if (first === undefined) return { kind: "none" };
  if (links.some(isOpenLink)) return { kind: "waiting" };
  const key = keyOf(links);
  if (key === handled) return { kind: "none" };
  if (others.length > 0) return { kind: "outcomes", key };
  const workspace = session.workspaces.find(
    (item) => item.id === first.workspace_id,
  );
  return workspace === undefined
    ? { kind: "settled", key }
    : { kind: "workspace", key, workspace };
}

/**
 * The key that the browser stores once the landing reached the
 * continuation, or null for no change. A continuation counts as handled
 * only when every named claim ended.
 */
export function continuationToRemember(
  continuation: ClaimContinuation,
): string | null {
  switch (continuation.kind) {
    case "outcomes":
    case "workspace":
    case "settled":
      return continuation.key;
    case "none":
    case "waiting":
      return null;
  }
}
