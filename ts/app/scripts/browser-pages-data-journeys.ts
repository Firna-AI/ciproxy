import assert from "node:assert/strict";

import { jobsHref } from "../src/view/jobs-links.ts";
import { savingsHref } from "../src/view/savings-report.ts";
import { control, facts, visible } from "./browser-fixture.ts";
import { apiAction, capturePage } from "./browser-pages-helpers.ts";
import { visibleJobDetails } from "./browser-pages-jobs-screens.ts";
import { visibleCostRows } from "./browser-pages-savings-screens.ts";
import { fixtureJobIds } from "./fixtures-jobs.ts";
import { workspaceId } from "./fixtures-core.ts";
import type { PagesSmokeRun } from "./browser-pages-runtime.ts";

export function dataJourneys(origin: string, output: string): PagesSmokeRun[] {
  return [
    {
      name: "history-filters-detail-and-back",
      async check(page, context, width) {
        await context.route("https://github.com/**", async (route) =>
          route.fulfill({
            status: 200,
            contentType: "text/html",
            body: '<!doctype html><link rel="icon" href="data:,"><h1>Fixture GitHub logs</h1>',
          }),
        );
        await control(origin, { scenario: "m21-jobs-history" });
        await page.goto(origin + jobsHref(workspaceId, "live"), {
          waitUntil: "domcontentloaded",
        });
        const history = page.getByRole("link", {
          name: "History",
          exact: true,
        });
        await visible(history);
        await history.focus();
        await history.press("Enter");
        await visible(
          page.getByRole("link", { name: "Browser checks", exact: true }),
        );
        await apiAction(page, "jobs/history", "GET", async () => {
          await page.getByLabel("Search jobs", { exact: true }).fill("Browser");
          await page.getByLabel("Search jobs", { exact: true }).press("Enter");
        });
        await apiAction(page, "jobs/history", "GET", () =>
          page.getByRole("radio", { name: "Boat", exact: true }).click(),
        );
        await page
          .getByRole("button", { name: "Finished: Today", exact: true })
          .click();
        await apiAction(page, "jobs/history", "GET", () =>
          page
            .getByRole("menuitem", { name: "Last 7 days", exact: true })
            .click(),
        );
        await visible(
          page.getByRole("link", { name: "Browser checks", exact: true }),
        );
        assert.ok(
          (await facts(origin)).some(
            (fact) =>
              fact.path.endsWith("/jobs/history") &&
              fact.query?.search?.[0] === "Browser" &&
              fact.query.place?.[0] === "boat" &&
              fact.query.from?.[0] === "2026-10-31",
          ),
          "History must send the chosen filters to the API.",
        );
        await capturePage(page, output, "history-filtered", width);
        await page
          .getByRole("link", { name: "Browser checks", exact: true })
          .click();
        await visible(
          page.getByRole("heading", { name: "Browser checks", exact: true }),
        );
        assert.equal(
          new URL(page.url()).searchParams.get("job"),
          fixtureJobIds.boat,
        );
        await visibleJobDetails(page);
        await capturePage(page, output, "history-boat-detail", width);
        const github = page.getByRole("link", {
          name: "View on GitHub",
          exact: true,
        });
        assert.equal(await github.getAttribute("target"), "_blank");
        const popup = page.waitForEvent("popup");
        await github.click();
        const logs = await popup;
        await visible(
          logs.getByRole("heading", {
            name: "Fixture GitHub logs",
            exact: true,
          }),
        );
        assert.equal(new URL(logs.url()).hostname, "github.com");
        await logs.close();
        await page
          .getByRole("navigation", { name: "Breadcrumb", exact: true })
          .getByRole("link", { name: "Jobs", exact: true })
          .click();
        await visible(page.getByLabel("Search jobs", { exact: true }));
        assert.equal(new URL(page.url()).searchParams.get("tab"), "history");
        await page.reload({ waitUntil: "domcontentloaded" });
        await visible(
          page.getByRole("link", { name: "Legacy request", exact: true }),
        );
      },
    },
    {
      name: "savings-range-navigation",
      async check(page, _context, width) {
        await control(origin, { scenario: "m21-savings-report" });
        await page.goto(origin + savingsHref(workspaceId, "last30_days"), {
          waitUntil: "domcontentloaded",
        });
        await visibleCostRows(page);
        for (const [label, range, amounts] of [
          ["This month", "this_month", ["$12.00", "$6.00", "$9.80"]],
          ["Last month", "last_month", ["$60.00", "$20.00", "$49.00"]],
        ] as const) {
          await apiAction(page, "savings", "GET", () =>
            page.getByRole("radio", { name: label, exact: true }).click(),
          );
          await visibleCostRows(page, [...amounts]);
          assert.equal(new URL(page.url()).searchParams.get("range"), range);
          assert.ok(
            (await facts(origin)).some(
              (fact) =>
                fact.path.endsWith("/savings") &&
                fact.query?.range?.[0] === range,
            ),
          );
          await capturePage(page, output, `savings-range-${range}`, width);
        }
        await page.reload({ waitUntil: "domcontentloaded" });
        await visibleCostRows(page, ["$60.00", "$20.00", "$49.00"]);
      },
    },
    {
      name: "savings-read-recovery",
      expectedHttpStatuses: [503],
      async check(page, _context, width) {
        await control(origin, { scenario: "m21-savings-error" });
        await page.goto(origin + savingsHref(workspaceId, "last30_days"), {
          waitUntil: "domcontentloaded",
        });
        await visible(
          page.getByRole("heading", {
            name: "Unable to load the savings report",
            exact: true,
          }),
        );
        await capturePage(page, output, "savings-recovery-error", width);
        await control(origin, { pages: { failures: [] } });
        await page
          .getByRole("button", { name: "Try again", exact: true })
          .click();
        await visibleCostRows(page);
      },
    },
  ];
}
