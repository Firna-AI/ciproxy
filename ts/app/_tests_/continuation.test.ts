import assert from "node:assert/strict";
import test from "node:test";

import { landingDecision } from "../src/routing/landing.ts";
import {
  browserOnlyReturn,
  completedReturn,
  pendingReturn,
  removedWithOldClaim,
  returnedInstallationId,
} from "../scripts/fixtures-returns.ts";
import {
  otherWorkspaceId,
  workspace,
  workspaceId,
} from "../scripts/fixtures-core.ts";

test("a verified completed install opens its exact workspace before a remembered workspace", () => {
  for (const priorRole of ["admin", "viewer"] as const) {
    assert.deepEqual(
      landingDecision(completedReturn(priorRole), {
        installStarted: true,
        preferredWorkspaceId: otherWorkspaceId,
        handledInstallationId: "9000",
      }),
      {
        landing: { workspace: workspace() },
        remember: returnedInstallationId,
        installOpen: false,
      },
    );
  }
});

test("a GitHub-started verified install is handled once in a ready session", () => {
  const current = completedReturn("viewer");
  for (const handledInstallationId of [null, "", "9000"]) {
    assert.deepEqual(
      landingDecision(current, {
        installStarted: false,
        preferredWorkspaceId: otherWorkspaceId,
        handledInstallationId,
      }),
      {
        landing: { workspace: workspace() },
        remember: returnedInstallationId,
        installOpen: false,
      },
    );
  }
  for (const installStarted of [false, true]) {
    assert.deepEqual(
      landingDecision(current, {
        installStarted,
        preferredWorkspaceId: otherWorkspaceId,
        handledInstallationId: returnedInstallationId,
      }),
      {
        landing: {
          workspace: workspace({
            id: otherWorkspaceId,
            name: "Orbit",
            role: "viewer",
          }),
        },
        remember: null,
        installOpen: false,
      },
    );
  }
});

test("a marked current return opens recovery before removed or ended access", () => {
  for (const state of ["app_removed", "access_ended"] as const) {
    assert.deepEqual(
      landingDecision(browserOnlyReturn(state), {
        installStarted: true,
        preferredWorkspaceId: workspaceId,
        handledInstallationId: "9001",
      }),
      {
        landing: { href: "/add-organization" },
        remember: null,
        installOpen: true,
      },
    );
  }
});

test("old masked claims retain the removed-App page without a marked return", () => {
  const old = removedWithOldClaim();
  old.setup = {
    kind: "organization",
    installation_id: "9001",
    github_id: "8001",
    login: "northstar-labs",
    suspended: false,
  };
  for (const handledInstallationId of [null, "", "9000", "9001"]) {
    assert.deepEqual(
      landingDecision(old, {
        installStarted: false,
        preferredWorkspaceId: otherWorkspaceId,
        handledInstallationId,
      }).landing,
      { href: "/app-removed" },
    );
  }
});

test("a pending install keeps its continuation until its exact destination opens", () => {
  const context = {
    installStarted: true,
    preferredWorkspaceId: otherWorkspaceId,
    handledInstallationId: "9000",
  };
  assert.deepEqual(landingDecision(pendingReturn(), context), {
    landing: { href: "/add-organization" },
    remember: null,
    installOpen: true,
  });
  assert.deepEqual(landingDecision(completedReturn("viewer"), context), {
    landing: { workspace: workspace() },
    remember: returnedInstallationId,
    installOpen: false,
  });
});
