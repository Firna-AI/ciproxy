import type { SessionView } from "../src/api/schema.ts";
import {
  claimId,
  link,
  otherClaimId,
  otherWorkspaceId,
  session,
  workspace,
  workspaceId,
} from "./fixtures-core.ts";
import { completedReturn } from "./fixtures-returns.ts";

export function lateApprovedReturn(): SessionView {
  return {
    ...completedReturn("viewer"),
    installation_continuation: [claimId],
    setup: { kind: "waiting_approval" },
  } satisfies SessionView;
}

export function lateApprovalWait(): SessionView {
  return session({
    state: "install_required",
    workspaces: [],
    setup: { kind: "waiting_approval" },
    reauthorization_url: "/api/auth/github",
    pending_install_requests: [
      {
        id: "91",
        github_id: "8003",
        organization: "northstar-labs",
        created_at: "2026-11-06T12:00:00Z",
      },
    ],
    organization_links: [
      link({
        id: otherClaimId,
        installation_id: "9000",
        state: "reauthorization_required",
        reauthorization_required: true,
        reauthorization_url: "/api/auth/github",
        workspaces: [],
      }),
    ],
  });
}

export function lateAddWait(): SessionView {
  const current = lateApprovedReturn();
  return {
    ...current,
    installation_continuation: [],
    organization_links: current.organization_links.filter(
      (claim) => claim.id === otherClaimId,
    ),
    pending_install_requests: lateApprovalWait().pending_install_requests,
  } satisfies SessionView;
}

export function multipleLateApprovals(): SessionView {
  const current = lateApprovedReturn();
  return {
    ...current,
    installation_continuation: [claimId, otherClaimId],
    workspaces: [
      workspace(),
      workspace({ id: otherWorkspaceId, name: "Orbit", role: "viewer" }),
    ],
    organization_links: current.organization_links.map((claim) => ({
      ...claim,
      organization:
        claim.workspace_id === workspaceId ? "northstar-labs" : "orbit-labs",
    })),
  } satisfies SessionView;
}
