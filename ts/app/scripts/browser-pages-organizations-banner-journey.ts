import assert from "node:assert/strict";

import { parsePageBannerView } from "../src/api/banner-reader.ts";
import { apiPrefix } from "../src/api/client.ts";
import { parseProblem, parseSuccess } from "../src/api/parse-session.ts";
import { routePath } from "../src/api/route-path.ts";
import { parseSettings } from "../src/api/settings-parse.ts";
import { billingHref, plansHref } from "../src/view/billing-routes.ts";
import { settingsHref } from "../src/view/settings-tabs.ts";
import type { BrowserResponse } from "./browser-driver.ts";
import { control, facts, insideViewport, visible } from "./browser-fixture.ts";
import { absent, capturePage } from "./browser-pages-helpers.ts";
import type { PagesSmokeRun } from "./browser-pages-runtime.ts";
import { workspaceId } from "./fixtures-core.ts";

function responseFor(path: string, method: string) {
  return (response: BrowserResponse) =>
    new URL(response.url()).pathname === path &&
    response.request().method() === method;
}

async function readBanner(response: BrowserResponse, organizations: boolean) {
  await response.finished();
  assert.equal(response.status(), 200);
  assert.equal(response.request().method(), "GET");
  assert.equal(
    new URL(response.url()).search,
    organizations ? "?page=organizations" : "",
  );
  const parsed = parsePageBannerView(await response.json());
  assert.ok(parsed !== undefined, "The browser banner response must parse.");
  return parsed.banner ?? null;
}

export function organizationsBannerJourney(
  origin: string,
  output: string,
): PagesSmokeRun {
  const params = { workspace_id: workspaceId };
  const bannerPath =
    apiPrefix + routePath("/workspaces/{workspace_id}/banner", params);
  const settingsPath =
    apiPrefix + routePath("/workspaces/{workspace_id}/settings", params);
  const organizations = settingsHref(workspaceId, "organizations");
  return {
    name: "organizations-api-banner-priority-and-turn-on",
    expectedHttpStatuses: [503],
    async check(page, _context, width) {
      await control(origin, {
        scenario: "m21-usage-payment-failed",
        pages: { organizationBanner: "plan" },
      });
      const firstBanner = page.waitForResponse(responseFor(bannerPath, "GET"));
      const firstSettings = page.waitForResponse(
        responseFor(settingsPath, "GET"),
      );
      await page.goto(origin + organizations, {
        waitUntil: "domcontentloaded",
      });
      assert.equal(
        (await readBanner(await firstBanner, true))?.kind,
        "payment_failed",
      );
      const settingsResponse = await firstSettings;
      await settingsResponse.finished();
      const initial = parseSettings(await settingsResponse.json());
      assert.ok(
        initial !== undefined,
        "The browser Settings response must parse.",
      );
      assert.deepEqual(initial.organizations[1]?.activation.reasons, [
        "admin_approval",
      ]);
      const rename = page.getByRole("button", { name: "Rename", exact: true });
      await visible(rename);
      const payment = page.getByText(
        "Stripe could not charge your card on 4 November",
        { exact: true },
      );
      await visible(payment);
      assert.equal(await page.getByRole("alert").count(), 1);
      await absent(page.getByText("orbit-labs is inactive", { exact: true }));
      await capturePage(page, output, "organizations-payment-priority", width);

      const tabs = page.getByRole("navigation", {
        name: "Settings tabs",
        exact: true,
      });
      const labelsBanner = page.waitForResponse(responseFor(bannerPath, "GET"));
      await tabs.getByRole("link", { name: "Labels", exact: true }).click();
      assert.equal(
        (await readBanner(await labelsBanner, false))?.kind,
        "payment_failed",
      );
      await visible(payment);
      const scopedAgain = page.waitForResponse(responseFor(bannerPath, "GET"));
      await tabs
        .getByRole("link", { name: "Organizations", exact: true })
        .click();
      assert.equal(
        (await readBanner(await scopedAgain, true))?.kind,
        "payment_failed",
      );
      await visible(rename);
      await visible(payment);
      assert.equal(await page.getByRole("alert").count(), 1);

      await control(origin, {
        pages: { usage: "over-organizations", organizationBanner: "plan" },
      });
      const planRead = page.waitForResponse(responseFor(bannerPath, "GET"));
      await page.goto(origin + organizations, {
        waitUntil: "domcontentloaded",
      });
      const plan = await readBanner(await planRead, true);
      assert.equal(plan?.kind, "inactive_organizations");
      assert.ok(plan?.kind === "inactive_organizations");
      assert.deepEqual(plan.organizations[0]?.activation.reasons, [
        "billing_upgrade",
      ]);
      const notice = page.getByRole("status");
      await visible(
        notice.getByText("orbit-labs is inactive", { exact: true }),
      );
      await visible(
        notice.getByText(
          "Free includes 1 organization. It starts its jobs on Team. Jobs keep running in the other organizations of this workspace.",
          { exact: true },
        ),
      );
      const usageLink = notice.getByRole("link", {
        name: "Plan and usage",
        exact: true,
      });
      const team = notice.getByRole("link", {
        name: "Choose Team",
        exact: true,
      });
      assert.equal(
        await usageLink.getAttribute("href"),
        billingHref(workspaceId),
      );
      assert.equal(await team.getAttribute("href"), plansHref(workspaceId));
      await insideViewport(notice, width);
      await capturePage(page, output, "organizations-team-offer", width);
      await team.click();
      await page.waitForURL(origin + plansHref(workspaceId));
      await visible(
        page.getByRole("heading", { name: "Business", exact: true }),
      );

      await control(origin, {
        pages: { usage: "paid", organizationBanner: "none" },
      });
      const noBanner = page.waitForResponse(responseFor(bannerPath, "GET"));
      const staleSettings = page.waitForResponse(
        responseFor(settingsPath, "GET"),
      );
      await page.goto(origin + organizations, {
        waitUntil: "domcontentloaded",
      });
      assert.equal(await readBanner(await noBanner, true), null);
      const staleResponse = await staleSettings;
      await staleResponse.finished();
      const stale = parseSettings(await staleResponse.json());
      assert.deepEqual(stale?.organizations[1]?.activation.reasons, [
        "admin_approval",
      ]);
      await visible(rename);
      await visible(page.getByText("Inactive", { exact: true }));
      await absent(page.getByText("orbit-labs is inactive", { exact: true }));
      assert.equal(await page.getByRole("status").count(), 0);
      await capturePage(page, output, "organizations-api-null", width);

      await control(origin, { pages: { organizationBanner: "enable-failed" } });
      const adminRead = page.waitForResponse(responseFor(bannerPath, "GET"));
      await page.goto(origin + organizations, {
        waitUntil: "domcontentloaded",
      });
      const admin = await readBanner(await adminRead, true);
      assert.ok(admin?.kind === "inactive_organizations");
      const inactive = admin.organizations[0];
      assert.ok(inactive !== undefined);
      assert.equal(inactive.activation.can_enable, true);
      assert.deepEqual(inactive.activation.reasons, ["admin_approval"]);
      const enablePath =
        apiPrefix +
        routePath(
          "/workspaces/{workspace_id}/organizations/{organization_id}/enable",
          { ...params, organization_id: inactive.id },
        );
      const enables = async () =>
        (await facts(origin)).filter(
          (fact) => fact.method === "POST" && fact.path === enablePath,
        );
      const turnOn = notice.getByRole("button", {
        name: "Turn on orbit-labs",
        exact: true,
      });
      await visible(turnOn);
      await visible(
        notice.getByText(
          "GitHub user 4783921, who added orbit-labs, is no longer an Admin. It starts its jobs after an Admin turns it on. Jobs keep running in the other organizations of this workspace.",
          { exact: true },
        ),
      );
      await insideViewport(notice, width);
      await turnOn.click();
      const dialog = page.getByRole("dialog", {
        name: "Turn on orbit-labs?",
        exact: true,
      });
      await visible(dialog);
      await visible(
        dialog.getByText(
          "The jobs of orbit-labs then run on the machines of this workspace, with its labels, settings, and overflow providers.",
          { exact: true },
        ),
      );
      await visible(
        dialog.getByText(
          "Ana Souza added orbit-labs on 1 September and is no longer an Admin.",
          { exact: true },
        ),
      );
      await capturePage(page, output, "organizations-turn-on-dialog", width);
      await dialog.getByRole("button", { name: "Cancel", exact: true }).click();
      await absent(dialog);
      assert.deepEqual(await enables(), []);

      await turnOn.click();
      const failed = page.waitForResponse(responseFor(enablePath, "POST"));
      await dialog
        .getByRole("button", { name: "Turn on", exact: true })
        .click();
      const failure = await failed;
      await failure.finished();
      assert.equal(failure.status(), 503);
      assert.equal(parseProblem(await failure.json())?.code, "unavailable");
      await visible(
        dialog.getByText("Unable to turn on the organization", { exact: true }),
      );
      await visible(
        dialog.getByText("ciproxy could not finish the change. Try again.", {
          exact: true,
        }),
      );
      assert.equal((await enables()).length, 1);
      await capturePage(page, output, "organizations-turn-on-failed", width);

      await control(origin, { pages: { organizationBanner: "admin" } });
      const accepted = page.waitForResponse(responseFor(enablePath, "POST"));
      const refreshed = page.waitForResponse(responseFor(bannerPath, "GET"));
      await dialog
        .getByRole("button", { name: "Turn on", exact: true })
        .click();
      const success = await accepted;
      await success.finished();
      assert.equal(success.status(), 200);
      assert.deepEqual(parseSuccess(await success.json()), { ok: true });
      assert.equal(await readBanner(await refreshed, true), null);
      await absent(dialog);
      await absent(page.getByText("orbit-labs is inactive", { exact: true }));
      await absent(page.getByText("Inactive", { exact: true }));
      assert.deepEqual(
        (await enables()).map((fact) => fact.status),
        [503, 200],
      );
      await capturePage(page, output, "organizations-turn-on-complete", width);
    },
  };
}
