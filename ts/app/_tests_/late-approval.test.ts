import assert from "node:assert/strict";
import test from "node:test";

import type { OrganizationLinkView } from "../src/api/schema.ts";
import { landingDecision } from "../src/routing/landing.ts";
import {
  claimId,
  otherWorkspaceId,
  workspace,
} from "../scripts/fixtures-core.ts";
import {
  lateApprovedReturn,
  multipleLateApprovals,
} from "../scripts/fixtures-late-approval.ts";

test("a plain sign-in opens the exact late-approved claim before an old Viewer workspace", () => {
  const current = lateApprovedReturn();
  const context = {
    installStarted: false,
    preferredWorkspaceId: otherWorkspaceId,
  };
  const open = {
    ...current,
    installation_continuation: [claimId],
    workspaces: current.workspaces.filter(
      (item) => item.id === otherWorkspaceId,
    ),
    organization_links: current.organization_links.map(
      (claim): OrganizationLinkView =>
        claim.id === claimId
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
  };
  const waiting = landingDecision(open, context);
  assert.deepEqual(waiting.landing, { href: "/add-organization" });
  assert.equal(waiting.rememberContinuation, null);
  const first = landingDecision(current, {
    ...context,
    handledContinuation: waiting.rememberContinuation,
  });
  assert.deepEqual(first.landing, {
    workspace: workspace(),
  });
  assert.equal(first.rememberContinuation, claimId);
  const ordinary = {
    workspace: workspace({
      id: otherWorkspaceId,
      name: "Orbit",
      role: "viewer",
    }),
  };
  const handled = landingDecision(current, {
    ...context,
    handledContinuation: first.rememberContinuation,
  });
  assert.deepEqual(handled.landing, ordinary);
  assert.equal(handled.rememberContinuation, null);
  assert.deepEqual(
    landingDecision({ ...current, installation_continuation: [] }, context)
      .landing,
    ordinary,
  );
});

test("multiple late-approved destinations open the return page without choosing by order", () => {
  const current = multipleLateApprovals();
  const context = {
    installStarted: false,
    preferredWorkspaceId: otherWorkspaceId,
  };
  const first = landingDecision(current, context);
  for (const continuation of [
    current.installation_continuation,
    [...current.installation_continuation].reverse(),
  ]) {
    const changedOrder = {
      ...current,
      installation_continuation: continuation,
    };
    const decision = landingDecision(changedOrder, context);
    assert.deepEqual(decision.landing, { href: "/add-organization" });
    assert.equal(decision.rememberContinuation, first.rememberContinuation);
    const handled = landingDecision(changedOrder, {
      ...context,
      handledContinuation: first.rememberContinuation,
    });
    assert.deepEqual(handled.landing, {
      workspace: workspace({
        id: otherWorkspaceId,
        name: "Orbit",
        role: "viewer",
      }),
    });
    assert.equal(handled.rememberContinuation, null);
  }
});
