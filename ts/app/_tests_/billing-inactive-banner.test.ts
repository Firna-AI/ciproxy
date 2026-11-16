import assert from "node:assert/strict";
import test from "node:test";

import { parsePageBannerView } from "../src/api/banner-reader.ts";
import type {
  InactiveOrganization,
  PageBannerView,
} from "../src/api/billing-types.ts";
import type { ActivationView } from "../src/api/schema.ts";
import { bannerModel } from "../src/view/banner.ts";
import { billingHref, plansHref } from "../src/view/billing-routes.ts";
import {
  activation,
  otherWorkspaceId,
  workspaceId,
} from "../scripts/fixtures-core.ts";

const workspace = { id: workspaceId, name: "Northstar" };
const organizationId = "40000000-0000-4000-8000-000000000002";

function inactiveModel(
  changes: Partial<ActivationView>,
  others: InactiveOrganization[] = [],
) {
  const organization: InactiveOrganization = {
    id: organizationId,
    login: "orbit-labs",
    activation: activation({ admitted: false, ...changes }),
  };
  const payload: PageBannerView = {
    banner: {
      kind: "inactive_organizations",
      organizations: [organization, ...others],
    },
  };
  const parsed = parsePageBannerView(payload);
  assert.ok(parsed !== undefined, "The inactive API banner must parse.");
  assert.ok(parsed.banner !== undefined && parsed.banner !== null);
  return bannerModel(parsed.banner, workspace, "settings");
}

test("the inactive API banner names its hold reasons and API identity", () => {
  const model = inactiveModel({
    reasons: ["billing_upgrade", "admin_approval"],
    added_by: "4783921",
    upgrade: { tier: "team", workspace_id: workspaceId },
  });
  assert.equal(model.title, "orbit-labs is inactive");
  assert.equal(
    model.body,
    "Free includes 1 organization, and GitHub user 4783921, who added orbit-labs, is no longer an Admin. It starts its jobs on Team, after an Admin turns it on. Jobs keep running in the other organizations of this workspace.",
  );
});

test("the inactive API banner keeps an absent adder unknown", () => {
  const model = inactiveModel({
    reasons: ["billing_upgrade", "admin_approval"],
    added_by: null,
    upgrade: { tier: "team", workspace_id: workspaceId },
  });
  assert.equal(
    model.body,
    "Free includes 1 organization, and the person who added orbit-labs is no longer an Admin. It starts its jobs on Team, after an Admin turns it on. Jobs keep running in the other organizations of this workspace.",
  );
});

test("several inactive API organizations keep their summary and no ambiguous Turn on", () => {
  const model = inactiveModel(
    { reasons: ["admin_approval"], can_enable: true },
    [
      {
        id: "40000000-0000-4000-8000-000000000003",
        login: "mokly-studio",
        activation: activation({
          admitted: false,
          reasons: ["billing_upgrade"],
          upgrade: { tier: "team", workspace_id: workspaceId },
        }),
      },
    ],
  );
  assert.equal(model.title, "2 organizations are inactive");
  assert.equal(
    model.body,
    "Each row says why. Jobs keep running in the other organizations of this workspace.",
  );
  assert.deepEqual(model.actions, [
    {
      kind: "link",
      label: "Plan and usage",
      href: billingHref(workspaceId),
      primary: false,
    },
    { kind: "link", label: "Choose Team", href: plansHref(workspaceId) },
  ]);
});

test("the inactive API banner offers only API permitted actions", () => {
  const usage = {
    kind: "link",
    label: "Plan and usage",
    href: billingHref(otherWorkspaceId),
    primary: false,
  };
  const team = {
    kind: "link",
    label: "Choose Team",
    href: plansHref(otherWorkspaceId),
  };
  const ask = { kind: "ask", text: "Ask an Admin" };
  const cases = [
    {
      name: "Admin with a plan hold",
      activation: {
        reasons: ["billing_upgrade"],
        upgrade: { tier: "team", workspace_id: otherWorkspaceId },
      },
      actions: [usage, team],
    },
    {
      name: "Admin with only an Admin hold",
      activation: { reasons: ["admin_approval"], can_enable: true },
      actions: [{ kind: "turn-on", label: "Turn on", organizationId }],
    },
    {
      name: "Billing with both holds",
      activation: {
        reasons: ["billing_upgrade", "admin_approval"],
        upgrade: { tier: "team", workspace_id: otherWorkspaceId },
        ask_admin: true,
      },
      actions: [usage, team, ask],
    },
    {
      name: "Viewer with both holds",
      activation: {
        reasons: ["billing_upgrade", "admin_approval"],
        ask_admin: true,
      },
      actions: [ask],
    },
  ] satisfies readonly {
    name: string;
    activation: Partial<ActivationView>;
    actions: readonly unknown[];
  }[];
  for (const entry of cases) {
    assert.deepEqual(
      inactiveModel(entry.activation).actions,
      entry.actions,
      entry.name,
    );
  }
});
