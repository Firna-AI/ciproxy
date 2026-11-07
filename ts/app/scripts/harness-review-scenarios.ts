import type { Scenario } from "./harness-scenarios.ts";
import { longWorkspaceName, workspace } from "./fixtures-core.ts";
import {
  browserOnlyReturn,
  completedReturn,
  pendingReturn,
  removedWithOldClaim,
} from "./fixtures-returns.ts";

export function applyReviewScenario(data: Scenario): void {
  if (data.name === "workspace-long") {
    data.session.workspaces = [workspace({ name: longWorkspaceName })];
  }
  if (
    [
      "root-install-completed",
      "root-install-viewer",
      "root-install-github",
      "root-install-handled",
      "root-explicit-workspace",
    ].includes(data.name)
  ) {
    data.session = completedReturn(
      data.name === "root-install-completed" ? "admin" : "viewer",
    );
  }
  if (data.name === "root-removed-old-claim")
    data.session = removedWithOldClaim();
  if (data.name === "root-removed-old-outcome") {
    data.session = removedWithOldClaim();
    data.session.setup = {
      kind: "organization",
      installation_id: "9001",
      github_id: "8001",
      login: "northstar-labs",
      suspended: false,
    };
  }
  if (data.name === "root-install-pending") data.session = pendingReturn();
  if (data.name === "root-reinstall-open")
    data.session = browserOnlyReturn("app_removed");
  if (data.name === "root-access-recovery")
    data.session = browserOnlyReturn("access_ended");
  const first = data.session.workspaces[0];
  if (first !== undefined && data.name === "session-refresh-no-role")
    first.role = null;
  if (first !== undefined && data.name === "session-refresh-workspace")
    first.role = "viewer";
  if (
    data.name.startsWith("claim-failure-") ||
    data.name === "claim-network-failure"
  ) {
    const member = data.session.workspaces[0];
    if (member !== undefined) {
      member.role = data.name === "claim-failure-workspace" ? "billing" : null;
      member.can_claim_admin = true;
    }
  }
}
