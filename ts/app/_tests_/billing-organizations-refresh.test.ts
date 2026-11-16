import assert from "node:assert/strict";
import test from "node:test";

import type { PageBanner } from "../src/api/billing-types.ts";
import { apiPrefix } from "../src/api/client.ts";
import { routePath } from "../src/api/route-path.ts";
import { bannerStore } from "../src/state/billing.ts";
import { watchOrganizationHolds } from "../src/state/organizations-banner.ts";
import { settingsServices } from "../src/state/settings-services.ts";
import { activation, session, workspaceId } from "../scripts/fixtures-core.ts";
import { bannerView, signedInWith, unavailable } from "./billing-support.ts";
import { held, settingsView } from "./settings-support.ts";
import { response, settled } from "./support.ts";

test("organization holds refresh the API banner after Turn on and a fresh entry", async () => {
  const organization = held(2, "orbit-labs", ["admin_approval"], workspaceId);
  const ready = settingsView({ organizations: [organization] });
  const enabled = settingsView({
    organizations: [
      { ...organization, activation: activation({ added_by: "4783921" }) },
    ],
  });
  const inactive: PageBanner = {
    kind: "inactive_organizations",
    organizations: [
      {
        id: organization.id,
        login: organization.login,
        activation: organization.activation,
      },
    ],
  };
  const { app, calls } = await signedInWith(
    "admin",
    response(bannerView(inactive)),
    response(unavailable, 503),
    response(ready),
    response({ ...ready, name: "New workspace name" }),
    response(unavailable, 503),
    response({ ok: true }),
    response(enabled),
    response(session()),
    response(bannerView(null)),
    response(ready),
    response(bannerView(inactive)),
    response(enabled),
    response(bannerView(null)),
  );
  const params = { workspace_id: workspaceId };
  const bannerPath =
    apiPrefix +
    routePath("/workspaces/{workspace_id}/banner", params) +
    "?page=organizations";
  const settingsPath =
    apiPrefix + routePath("/workspaces/{workspace_id}/settings", params);
  const enablePath =
    apiPrefix +
    routePath(
      "/workspaces/{workspace_id}/organizations/{organization_id}/enable",
      {
        ...params,
        organization_id: organization.id,
      },
    );
  const settings = settingsServices(app).settings(workspaceId);
  const banner = bannerStore(app, workspaceId, "organizations");
  const requests = (from = 0) =>
    calls.slice(from).map((call) => [call.init.method, call.url]);
  const stop = watchOrganizationHolds(app, workspaceId);
  try {
    await settled();
    assert.deepEqual(requests(), [["GET", bannerPath]]);
    assert.deepEqual(banner.state.get(), {
      status: "ready",
      banner: inactive,
      refreshFailure: null,
    });

    await settings.load();
    assert.equal(settings.state.get().status, "failed");
    await settings.load();
    await settings.load();
    await settings.load();
    await settled();
    assert.deepEqual(requests(), [
      ["GET", bannerPath],
      ["GET", settingsPath],
      ["GET", settingsPath],
      ["GET", settingsPath],
      ["GET", settingsPath],
    ]);

    const beforeChange = calls.length;
    assert.deepEqual(await settings.turnOn(organization.id), {
      ok: true,
      value: { ok: true },
    });
    await settled();
    assert.deepEqual(requests(beforeChange), [
      ["POST", enablePath],
      ["GET", settingsPath],
      ["GET", "/api/session"],
      ["GET", bannerPath],
    ]);
    assert.deepEqual(banner.state.get(), {
      status: "ready",
      banner: null,
      refreshFailure: null,
    });

    stop();
    const beforeStopped = calls.length;
    await settings.load();
    await settled();
    assert.deepEqual(requests(beforeStopped), [["GET", settingsPath]]);

    const beforeEntry = calls.length;
    const stopAgain = watchOrganizationHolds(app, workspaceId);
    try {
      await settled();
      assert.deepEqual(requests(beforeEntry), [["GET", bannerPath]]);
      assert.deepEqual(banner.state.get(), {
        status: "ready",
        banner: inactive,
        refreshFailure: null,
      });
      const beforeCachedChange = calls.length;
      await settings.load();
      await settled();
      assert.deepEqual(requests(beforeCachedChange), [
        ["GET", settingsPath],
        ["GET", bannerPath],
      ]);
      assert.deepEqual(banner.state.get(), {
        status: "ready",
        banner: null,
        refreshFailure: null,
      });
    } finally {
      stopAgain();
    }
  } finally {
    stop();
  }
});
