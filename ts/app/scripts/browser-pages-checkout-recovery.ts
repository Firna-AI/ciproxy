import assert from "node:assert/strict";

import type { PlanSelection } from "../src/api/billing-types.ts";
import { checkoutHref } from "../src/view/billing-routes.ts";
import { control, facts, visible } from "./browser-fixture.ts";
import { absent, apiAction, capturePage } from "./browser-pages-helpers.ts";
import { workspaceId } from "./fixtures-core.ts";
import type { PagesSmokeRun } from "./browser-pages-runtime.ts";

export function checkoutRecoveryRuns(
  origin: string,
  output: string,
): PagesSmokeRun[] {
  return [
    {
      name: "checkout-billing-read-recovery",
      expectedHttpStatuses: [503],
      async check(page, _context, width) {
        const selection: PlanSelection = {
          tier: "team",
          add_ons: { start_blocks: 2, machines: 1 },
        };
        await control(origin, {
          scenario: "m21-review-free",
          pages: { failures: ["billing"] },
        });
        const reviewed = page.waitForResponse(/\/checkout\/review\?/);
        const prerequisite = page.waitForResponse(
          (response) =>
            new URL(response.url()).pathname ===
              `/api/workspaces/${workspaceId}/billing` &&
            response.request().method() === "GET",
        );
        await page.goto(origin + checkoutHref(workspaceId, selection), {
          waitUntil: "domcontentloaded",
        });
        const [reviewReply, statusReply] = await Promise.all([
          reviewed,
          prerequisite,
        ]);
        await Promise.all([reviewReply.finished(), statusReply.finished()]);
        assert.equal(
          reviewReply.status(),
          200,
          "The exact review must succeed.",
        );
        assert.equal(
          statusReply.status(),
          503,
          "Only the billing prerequisite must fail.",
        );
        await visible(page.getByLabel("Extra starts: 2", { exact: true }));
        await visible(
          page.getByLabel("Extra own machines: 1", { exact: true }),
        );
        const summary = page.getByRole("region", {
          name: "Summary",
          exact: true,
        });
        await visible(
          summary.getByText("Unable to load plan and usage", { exact: true }),
        );
        await visible(
          summary.getByText(
            "Payments wait until it loads. Jobs keep running.",
            { exact: true },
          ),
        );
        await absent(
          summary.getByRole("button", {
            name: "Continue to payment",
            exact: true,
          }),
        );
        assert.equal(
          (await facts(origin)).filter(
            (fact) => fact.kind === "api" && fact.method !== "GET",
          ).length,
          0,
        );
        await capturePage(page, output, "checkout-billing-failure", width);
        await control(origin, { pages: { failures: [] } });
        await apiAction(page, "billing", "GET", () =>
          summary
            .getByRole("button", { name: "Try again", exact: true })
            .click(),
        );
        const pay = summary.getByRole("button", {
          name: "Continue to payment",
          exact: true,
        });
        await visible(pay);
        assert.equal(await pay.isEnabled(), true);
        await visible(page.getByLabel("Extra starts: 2", { exact: true }));
        await visible(
          page.getByLabel("Extra own machines: 1", { exact: true }),
        );
        await visible(summary.getByText("$89 a month", { exact: true }));
        assert.equal(
          (await facts(origin)).filter(
            (fact) => fact.kind === "api" && fact.method !== "GET",
          ).length,
          0,
          "Recovery must not start a payment.",
        );
        await capturePage(page, output, "checkout-billing-recovered", width);
        await pay.click();
        await visible(
          page.getByRole("heading", {
            name: "Fixture navigation complete",
            exact: true,
          }),
        );
        const payments = (await facts(origin)).filter(
          (fact) => fact.method === "POST" && fact.path.endsWith("/checkout"),
        );
        assert.equal(payments.length, 1);
        assert.deepEqual(payments[0]?.body, selection);
      },
    },
  ];
}
