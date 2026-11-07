import type {
  MachineView,
  OrganizationLinkView,
  SessionView,
  SetupView,
} from "../src/api/schema.ts";
import {
  activation,
  link,
  otherWorkspaceId,
  session,
  workspace,
  workspaceId,
} from "./fixtures-core.ts";
import { job, machine, setup } from "./fixtures-setup.ts";
import { applyReviewScenario } from "./harness-review-scenarios.ts";

export const scenarios = [
  "signed-out",
  "install",
  "install-pending",
  "install-reauthorize",
  "personal",
  "app-removed",
  "access-ended",
  "no-role",
  "no-role-claim",
  "admin",
  "billing",
  "viewer",
  "billing-claim",
  "viewer-claim",
  "soft-limit",
  "links-picker",
  "links-request",
  "links-recovery",
  "links-waiting",
  "links-confirming",
  "links-linking",
  "links-inactive-plan",
  "links-inactive-admin",
  "links-inactive-both",
  "links-preparing",
  "setup-waiting",
  "setup-connected",
  "setup-no-kvm",
  "setup-available",
  "setup-assigned",
  "setup-started",
  "setup-success",
  "setup-failure",
  "setup-load-failure",
  "command-load-failure",
  "command-revoke-failure",
  "command-expired",
  "root-install-completed",
  "root-install-viewer",
  "root-install-github",
  "root-install-handled",
  "root-install-pending",
  "root-explicit-workspace",
  "root-removed-old-claim",
  "root-removed-old-outcome",
  "root-reinstall-open",
  "root-access-recovery",
  "session-refresh-no-role",
  "session-refresh-workspace",
  "claim-failure-no-role",
  "claim-failure-workspace",
  "claim-failure-csrf",
  "claim-network-failure",
  "workspace-long",
] as const;

export type ScenarioName = (typeof scenarios)[number];

export function scenarioName(value: unknown): ScenarioName | undefined {
  return scenarios.find((name) => name === value);
}

export type Scenario = {
  name: ScenarioName;
  signedOut: boolean;
  session: SessionView;
  setup: SetupView;
  machines: MachineView[];
  setupFails: boolean;
  commandFails: boolean;
  revokeFails: boolean;
  linkedOutcome?: OrganizationLinkView;
};

function linkFor(name: ScenarioName): OrganizationLinkView {
  const base = link();
  switch (name) {
    case "links-recovery":
      return link({
        state: "reauthorization_required",
        reauthorization_required: true,
      });
    case "links-waiting":
      return link({ state: "waiting_for_admin", workspaces: [] });
    case "links-confirming":
      return link({ state: "confirming_installation", workspaces: [] });
    case "links-linking":
      return link({ state: "linking", workspace_id: workspaceId });
    case "links-inactive-plan":
    case "links-inactive-admin":
    case "links-inactive-both":
    case "links-preparing": {
      const billing =
        name === "links-inactive-plan" || name === "links-inactive-both";
      const admin =
        name === "links-inactive-admin" || name === "links-inactive-both";
      return link({
        state: "linked_inactive",
        workspace_id: workspaceId,
        activation: activation({
          admitted: false,
          preparing: name === "links-preparing",
          reasons: [
            ...(billing ? ["billing_upgrade" as const] : []),
            ...(admin ? ["admin_approval" as const] : []),
          ],
          upgrade: billing ? { tier: "team", workspace_id: workspaceId } : null,
          can_enable: admin && !billing,
        }),
      });
    }
    default:
      return base;
  }
}

export function recordedScenario(name: ScenarioName): Scenario {
  const data: Scenario = {
    name,
    signedOut: name === "signed-out",
    session: session(),
    setup: setup(),
    machines: [],
    setupFails: name === "setup-load-failure",
    commandFails: name === "command-load-failure",
    revokeFails: name === "command-revoke-failure",
  };
  const current = workspace();
  const other = workspace({ id: otherWorkspaceId, name: "Orbit", role: null });
  data.session.workspaces = [current, other];
  if (name === "billing" || name === "billing-claim") current.role = "billing";
  if (name === "viewer" || name === "viewer-claim") current.role = "viewer";
  if (name === "no-role" || name === "no-role-claim") current.role = null;
  current.can_claim_admin = name.endsWith("-claim");
  current.organization_limit_exceeded = name === "soft-limit";
  if (name.startsWith("install") || name === "personal") {
    data.session.state = "install_required";
    data.session.workspaces = [];
  }
  if (name === "install-pending" || name === "links-request") {
    data.session.setup = { kind: "waiting_approval" };
    data.session.pending_install_requests = [
      {
        id: "91",
        github_id: "8001",
        organization: "orbit-labs",
        created_at: "2026-11-06T12:00:00Z",
      },
    ];
  }
  if (name === "install-reauthorize") {
    data.session.reauthorization_required = true;
    data.session.reauthorization_url = "/api/auth/github?fixture=reauthorize";
  }
  if (name === "personal") {
    data.session.setup = {
      kind: "organizations_only",
      installation_id: "9002",
      remove_url: "https://github.com/settings/installations/9002",
    };
  }
  if (name === "app-removed") {
    data.session.state = "app_removed";
    data.session.workspaces = [];
    data.session.removed_organization = {
      github_id: "8001",
      login: "northstar-labs",
      delete_after: "2026-12-06T12:00:00Z",
    };
  }
  if (name === "access-ended") {
    data.session.state = "access_ended";
    data.session.workspaces = [];
  }
  if (name.startsWith("links-") && name !== "links-request") {
    const recorded = linkFor(name);
    if (recorded.state === "linked_inactive") {
      data.session.organization_links = [link()];
      data.linkedOutcome = recorded;
    } else data.session.organization_links = [recorded];
  }
  if (
    [
      "setup-connected",
      "setup-available",
      "setup-assigned",
      "setup-started",
      "setup-success",
      "setup-failure",
    ].includes(name)
  ) {
    data.machines = [machine()];
    data.setup.machines = 1;
    data.setup.waiting_for_machine = false;
  }
  if (name === "setup-no-kvm") {
    data.machines = [machine({ kvm: false })];
    data.setup.machines = 1;
  }
  if (name === "setup-available")
    data.setup.first_job = job({
      phase: "available",
      assigned_at: null,
      started_at: null,
    });
  if (name === "setup-assigned")
    data.setup.first_job = job({ phase: "assigned", started_at: null });
  if (name === "setup-started") data.setup.first_job = job();
  if (name === "setup-success" || name === "setup-failure") {
    data.setup.first_job = job({
      phase: "completed",
      result: name === "setup-success" ? "success" : "failure",
      completed_at: "2026-11-06T12:07:22Z",
    });
  }
  applyReviewScenario(data);
  return data;
}
