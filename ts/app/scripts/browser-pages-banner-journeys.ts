import assert from "node:assert/strict";

import { apiPrefix } from "../src/api/client.ts";
import { routePath } from "../src/api/route-path.ts";
import {
  billingHref,
  checkoutHref,
  plansHref,
} from "../src/view/billing-routes.ts";
import { jobsHref } from "../src/view/jobs-links.ts";
import { control, facts, visible } from "./browser-fixture.ts";
import { absent, capturePage } from "./browser-pages-helpers.ts";
import { visibleCostRows } from "./browser-pages-savings-screens.ts";
import { baseSelection } from "./fixtures-billing.ts";
import { workspaceId } from "./fixtures-core.ts";
import type { PagesSmokeRun } from "./browser-pages-runtime.ts";

export function bannerJourneys(
  origin: string,
  output: string,
): PagesSmokeRun[] {
  return [
    {
      name: "payment-banner-priority-across-navigation",
      async check(page, _context, width) {
        await control(origin, { scenario: "m21-usage-payment-failed" });
        await page.goto(origin + jobsHref(workspaceId, "live"), {
          waitUntil: "domcontentloaded",
        });
        const banner = page.getByText(
          "Stripe could not charge your card on 4 November",
          { exact: true },
        );
        await visible(banner);
        assert.equal(await page.getByRole("alert").count(), 1);
        await absent(
          page.getByText("Northstar passed a limit of its plan", {
            exact: true,
          }),
        );
        await capturePage(page, output, "banner-priority-jobs", width);
        const main = page.getByRole("navigation", {
          name: "Main",
          exact: true,
        });
        await main.getByRole("link", { name: "Savings", exact: true }).click();
        await visibleCostRows(page);
        await visible(banner);
        assert.equal(await page.getByRole("alert").count(), 1);
        await absent(
          page.getByText("Northstar passed a limit of its plan", {
            exact: true,
          }),
        );
        await capturePage(page, output, "banner-priority-savings", width);
        await main
          .getByRole("link", {
            name: width < 640 ? "Plan" : "Plan and usage",
            exact: true,
          })
          .click();
        await visible(
          page.getByRole("heading", { name: "This period", exact: true }),
        );
        await visible(banner);
        assert.equal(await page.getByRole("alert").count(), 1);
        await absent(
          page.getByText("Northstar passed a limit of its plan", {
            exact: true,
          }),
        );
        await capturePage(page, output, "banner-priority-usage", width);
        const bannerPath =
          apiPrefix +
          routePath("/workspaces/{workspace_id}/banner", {
            workspace_id: workspaceId,
          });
        assert.deepEqual(
          (await facts(origin)).filter(
            (fact) =>
              fact.kind === "api" &&
              fact.method === "GET" &&
              fact.path === bannerPath,
          ),
          [],
          "Native Jobs, Savings, and Plan and usage visits must send no GET /banner.",
        );
      },
    },
    {
      name: "viewer-cannot-use-paid-controls",
      async check(page, _context, width) {
        await control(origin, { scenario: "m21-viewer" });
        await page.goto(origin + billingHref(workspaceId), {
          waitUntil: "domcontentloaded",
        });
        await visible(
          page.getByRole("heading", { name: "This period", exact: true }),
        );
        await absent(
          page.getByRole("button", { name: "Invoices and card", exact: true }),
        );
        await absent(
          page.getByRole("button", { name: "Save add-ons", exact: true }),
        );
        await absent(
          page.getByRole("button", { name: "More: Extra starts", exact: true }),
        );
        await capturePage(page, output, "viewer-paid-usage", width);
        await page.goto(origin + plansHref(workspaceId), {
          waitUntil: "domcontentloaded",
        });
        await visible(
          page.getByRole("heading", { name: "Business", exact: true }),
        );
        await absent(
          page.getByRole("link", { name: "Choose Business", exact: true }),
        );
        await absent(
          page.getByRole("button", { name: "Move to Free", exact: true }),
        );
        await capturePage(page, output, "viewer-plan-controls", width);
        await page.goto(
          origin + checkoutHref(workspaceId, baseSelection("business")),
          { waitUntil: "domcontentloaded" },
        );
        await visible(
          page.getByRole("heading", {
            name: "Ask an Admin or a Billing person",
            exact: true,
          }),
        );
        await absent(
          page.getByRole("button", { name: "Move to Business", exact: true }),
        );
      },
    },
  ];
}
