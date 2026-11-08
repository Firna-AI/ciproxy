import type { SessionView, WorkspaceRole } from "../src/api/schema.ts";
import {
  activation,
  link,
  otherClaimId,
  otherWorkspaceId,
  session,
  workspace,
  workspaceId,
} from "./fixtures-core.ts";

export const returnedInstallationId = "9003";

/** A verified completed direct install, with an older workspace first. */
export function completedReturn(
  priorRole: WorkspaceRole = "admin",
): SessionView {
  return session({
    setup: {
      kind: "organization",
      github_id: "8003",
      installation_id: returnedInstallationId,
      login: "northstar-labs",
      suspended: false,
    },
    workspaces: [
      workspace({ id: otherWorkspaceId, name: "Orbit", role: priorRole }),
      workspace(),
    ],
    organization_links: [
      link({
        id: otherClaimId,
        installation_id: "9000",
        state: "linked",
        workspace_id: otherWorkspaceId,
        activation: activation(),
      }),
      link({
        installation_id: returnedInstallationId,
        state: "linked",
        workspace_id: workspaceId,
        organization_id: "fixture-new-registration",
        activation: activation(),
      }),
    ],
  });
}

export function pendingReturn(): SessionView {
  const current = completedReturn("viewer");
  return {
    ...current,
    workspaces: current.workspaces.filter(
      (item) => item.id === otherWorkspaceId,
    ),
    organization_links: current.organization_links.map((claim) =>
      claim.installation_id === returnedInstallationId
        ? {
            ...claim,
            state: "confirming_installation",
            workspace_id: null,
            organization_id: null,
            activation: null,
            workspaces: [],
          }
        : claim,
    ),
  } satisfies SessionView;
}

/** The real browser-only API view masks even an older completed claim. */
export function removedWithOldClaim(): SessionView {
  return session({
    state: "app_removed",
    workspaces: [],
    setup: null,
    removed_organization: {
      github_id: "8001",
      login: "northstar-labs",
      delete_after: "2026-12-06T12:00:00Z",
    },
    reauthorization_url: "/api/auth/github",
    organization_links: [
      link({
        state: "reauthorization_required",
        reauthorization_required: true,
        reauthorization_url: "/api/auth/github",
        workspaces: [],
        workspace_id: null,
        organization_id: null,
        activation: null,
      }),
    ],
  });
}

/** A fresh verified return can identify its exact browser-only claim. */
export function browserOnlyReturn(
  state: "app_removed" | "access_ended",
): SessionView {
  const recorded = removedWithOldClaim();
  const verified = completedReturn();
  return {
    ...recorded,
    state,
    removed_organization:
      state === "app_removed" ? recorded.removed_organization : null,
    setup: verified.setup,
    organization_links: recorded.organization_links.map((claim) => ({
      ...claim,
      installation_id: returnedInstallationId,
    })),
  } satisfies SessionView;
}
