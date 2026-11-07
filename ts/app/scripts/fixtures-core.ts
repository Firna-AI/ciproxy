// Recorded dashboard responses. Every body uses the generated Rust contract.
import type {
  ActivationView,
  EligibleWorkspace,
  OrganizationLinkView,
  SessionView,
  WorkspaceView,
} from "../src/api/schema.ts";

export const fixtureTime = "2026-11-06T12:00:00Z";
export const workspaceId = "10000000-0000-4000-8000-000000000001";
export const otherWorkspaceId = "10000000-0000-4000-8000-000000000002";
export const claimId = "20000000-0000-4000-8000-000000000001";
export const otherClaimId = "20000000-0000-4000-8000-000000000002";
export const longWorkspaceName = "centralized-infrastructure-automation-x";

export function workspace(
  overrides: Partial<WorkspaceView> = {},
): WorkspaceView {
  return {
    admins: ["mira", "sasha"],
    can_claim_admin: false,
    id: workspaceId,
    name: "Northstar",
    organization_limit_exceeded: false,
    role: "admin",
    ...overrides,
  } satisfies WorkspaceView;
}

export function activation(
  overrides: Partial<ActivationView> = {},
): ActivationView {
  return {
    added_by: "mira",
    admitted: true,
    ask_admin: false,
    can_enable: false,
    preparing: false,
    reasons: [],
    upgrade: null,
    ...overrides,
  } satisfies ActivationView;
}

export function offers(): EligibleWorkspace[] {
  return [
    { id: workspaceId, name: "Northstar", requires_upgrade: true },
    { id: otherWorkspaceId, name: "Orbit", requires_upgrade: false },
  ] satisfies EligibleWorkspace[];
}

export function link(
  overrides: Partial<OrganizationLinkView> = {},
): OrganizationLinkView {
  return {
    activation: null,
    id: claimId,
    installation_id: "9001",
    organization: "northstar-labs",
    organization_id: null,
    reauthorization_required: false,
    reauthorization_url: "/api/auth/github?fixture=claim-reauthorize",
    state: "choose_workspace",
    workspace_id: null,
    workspaces: offers(),
    ...overrides,
  } satisfies OrganizationLinkView;
}

export function session(overrides: Partial<SessionView> = {}): SessionView {
  return {
    csrf_token: "fixture-csrf-current",
    expires_at: "2026-12-06T12:00:00Z",
    installation_continuation: [],
    install_url: "/__fixture/github/install",
    login: "mira",
    organization_links: [],
    pending_install_requests: [],
    reauthorization_required: false,
    reauthorization_url: null,
    removed_organization: null,
    setup: null,
    state: "ready",
    workspaces: [workspace()],
    ...overrides,
  } satisfies SessionView;
}
