import assert from "node:assert/strict";
import test from "node:test";

import type { ApiResult } from "../src/api/client.ts";
import type { PageBanner, PageBannerView } from "../src/api/billing-types.ts";
import { parsePageBannerView } from "../src/api/banner-reader.ts";
import { bannerStore } from "../src/state/billing.ts";
import { createBannerStore } from "../src/state/billing-banner.ts";
import { bannerModel } from "../src/view/banner.ts";
import { billingHref, billingPaths } from "../src/view/billing-routes.ts";
import { workspaceId } from "../scripts/fixtures-core.ts";
import { bannerView, teamOffer, unavailable } from "./billing-support.ts";
import { appWith, deferred, response, settled } from "./support.ts";

const workspace = { id: workspaceId, name: "mokly-ai" };

const failed: PageBanner = {
  kind: "payment_failed",
  action: "update_card",
  failed_at: "2026-12-04T09:00:00Z",
  next_retry_at: null,
};

test("each banner gives the payer its action and every other role who to ask", () => {
  const cases: [PageBanner, string][] = [
    [failed, "portal"],
    [{ ...failed, action: "ask_admin_or_billing" }, "ask"],
    [
      {
        kind: "subscription_ended",
        action: "choose_plan",
        until: "2027-01-18T00:00:00Z",
      },
      "link:/plans",
    ],
    [
      { kind: "soft_limit", action: "choose_plan", offer: teamOffer },
      "link:/billing",
    ],
    [
      { kind: "soft_limit", action: "ask_admin_or_billing", offer: teamOffer },
      "ask",
    ],
  ];
  for (const [banner, expected] of cases) {
    const [action] = bannerModel(banner, workspace, "jobs").actions;
    const shown =
      action?.kind === "link"
        ? `link:${action.href.split("?")[0] ?? ""}`
        : action?.kind;
    assert.equal(shown, expected, `${banner.kind} ${JSON.stringify(banner)}`);
  }
  const onBilling = bannerModel(
    { kind: "soft_limit", action: "choose_plan", offer: teamOffer },
    workspace,
    "billing",
  ).actions[0];
  assert.equal(
    onBilling?.kind === "link" && onBilling.href.split("?")[0],
    "/plans",
  );
  // On Plans itself, a link to Plans would only reload the page, so it opens
  // Plan and usage. Who to ask and the billing portal stay.
  const onPlans = (banner: PageBanner) =>
    bannerModel(banner, workspace, "billing", billingPaths.plans).actions[0];
  const soft: PageBanner = {
    kind: "soft_limit",
    action: "choose_plan",
    offer: teamOffer,
  };
  const endedPlan: PageBanner = {
    kind: "subscription_ended",
    action: "choose_plan",
    until: "2027-01-18T00:00:00Z",
  };
  for (const banner of [soft, endedPlan]) {
    assert.deepEqual(
      onPlans(banner),
      { kind: "link", label: "Plan and usage", href: billingHref(workspaceId) },
      banner.kind,
    );
  }
  assert.equal(onPlans(failed)?.kind, "portal");
  assert.equal(
    onPlans({ ...soft, action: "ask_admin_or_billing" })?.kind,
    "ask",
  );
});

test("a banner names only the dates that the API gives", () => {
  const dated = bannerModel(failed, workspace, "jobs");
  assert.match(dated.title, /4\u00a0December/);
  assert.doesNotMatch(dated.body, /\d/);
  const ended = bannerModel(
    {
      kind: "subscription_ended",
      action: "choose_plan",
      until: "2027-01-18T00:00:00Z",
    },
    workspace,
    "jobs",
  );
  assert.doesNotMatch(
    `${ended.title} ${ended.body}`,
    /\d|January|Team|Business/,
  );
});

test("the banner reader rejects a kind or an action outside the contract", () => {
  assert.equal(
    parsePageBannerView({ banner: { kind: "trial_ended" } }),
    undefined,
  );
  assert.equal(
    parsePageBannerView({ banner: { ...failed, action: "pay_now" } }),
    undefined,
  );
  assert.deepEqual(parsePageBannerView({}), { banner: null });
  assert.deepEqual(parsePageBannerView({ banner: failed }), { banner: failed });
});

test("a page change reads the banner unless a read just ended", async () => {
  const { app, calls, setNow } = appWith(
    response(bannerView(failed)),
    response(unavailable, 503),
    response(bannerView(null)),
  );
  const store = bannerStore(app, workspaceId);
  await store.visit();
  await store.visit();
  assert.equal(calls.length, 1);
  assert.equal(calls[0]?.url, `/api/workspaces/${workspaceId}/banner`);
  setNow(Date.parse("2026-11-06T12:00:03Z"));
  await store.visit();
  const kept = store.state.get();
  assert.equal(kept.status === "ready" && kept.banner?.kind, "payment_failed");
  assert.equal(kept.status === "ready" && kept.refreshFailure?.kind, "problem");
  await store.refresh();
  assert.deepEqual(store.state.get(), {
    status: "ready",
    banner: null,
    refreshFailure: null,
  });
});

test("a failed first read shows the failure, and a newer read wins over an older one", async () => {
  const first = deferred<ApiResult<PageBannerView>>();
  const second = deferred<ApiResult<PageBannerView>>();
  const replies = [first.promise, second.promise];
  const store = createBannerStore({
    read: () => replies.shift() ?? Promise.reject(new Error("No reply")),
    report: (result) => result,
    now: () => 0,
  });
  void store.refresh();
  const newer = store.refresh();
  second.resolve({ ok: true, value: { banner: null } });
  await newer;
  first.resolve({ ok: true, value: { banner: failed } });
  await settled();
  assert.deepEqual(store.state.get(), {
    status: "ready",
    banner: null,
    refreshFailure: null,
  });
  const failing = createBannerStore({
    read: async () => ({ ok: false, failure: { kind: "network" } }),
    report: (result) => result,
    now: () => 0,
  });
  await failing.refresh();
  assert.deepEqual(failing.state.get(), {
    status: "failed",
    failure: { kind: "network" },
  });
});

test("the Organizations scope asks for its own banner", async () => {
  const { app, calls } = appWith(response(bannerView()));
  await bannerStore(app, workspaceId, "organizations").visit();
  assert.equal(
    calls[0]?.url,
    `/api/workspaces/${workspaceId}/banner?page=organizations`,
  );
});
