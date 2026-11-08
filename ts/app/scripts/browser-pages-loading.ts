import {
  billingHref,
  checkoutHref,
  plansHref,
} from "../src/view/billing-routes.ts";
import { jobHref, jobsHref } from "../src/view/jobs-links.ts";
import { savingsHref } from "../src/view/savings-report.ts";
import { control, visible } from "./browser-fixture.ts";
import { absent, capturePage } from "./browser-pages-helpers.ts";
import { baseSelection } from "./fixtures-billing.ts";
import { workspaceId } from "./fixtures-core.ts";
import { fixtureJobIds } from "./fixtures-jobs.ts";
import type { PageRoute } from "./harness-pages-state.ts";
import type { PagesSmokeRun } from "./browser-pages-runtime.ts";

export function loadingRuns(origin: string, output: string): PagesSmokeRun[] {
  const cases: {
    name: string;
    route: PageRoute;
    path: string;
    heading: string;
  }[] = [
    {
      name: "jobs-loading",
      route: "jobs/live",
      path: jobsHref(workspaceId, "live"),
      heading: "Jobs",
    },
    {
      name: "history-loading",
      route: "jobs/history",
      path: jobsHref(workspaceId, "history"),
      heading: "Jobs",
    },
    {
      name: "job-loading",
      route: "jobs/detail",
      path: jobHref(workspaceId, fixtureJobIds.machine),
      heading: "Job",
    },
    {
      name: "savings-loading",
      route: "savings",
      path: savingsHref(workspaceId, "last30_days"),
      heading: "Savings",
    },
    {
      name: "usage-loading",
      route: "billing",
      path: billingHref(workspaceId),
      heading: "Plan and usage",
    },
    {
      name: "plans-loading",
      route: "plans",
      path: plansHref(workspaceId),
      heading: "Plans",
    },
    {
      name: "review-loading",
      route: "checkout/review",
      path: checkoutHref(workspaceId, baseSelection("business")),
      heading: "Move to Business",
    },
  ];
  return cases.map((item) => ({
    name: item.name,
    cleanup: () => control(origin, { pages: { held: [] } }),
    async check(page, _context, width) {
      await control(origin, {
        scenario: "m21-usage-paid",
        pages: {
          held:
            item.name === "review-loading"
              ? ["billing", item.route]
              : [item.route],
        },
      });
      const route =
        item.route === "jobs/detail"
          ? `jobs/${fixtureJobIds.machine}`
          : item.route;
      const pending = page.waitForRequest(
        new RegExp(`/api/workspaces/[^/]+/${route}(?:\\?|$)`),
      );
      const checking =
        item.name === "review-loading"
          ? page.waitForRequest(/\/billing$/)
          : undefined;
      await page.goto(origin + item.path, { waitUntil: "domcontentloaded" });
      await pending;
      await visible(
        page.getByRole("heading", { name: item.heading, exact: true }),
      );
      if (item.name === "review-loading") {
        await checking;
        await visible(page.getByText("Checking your plan", { exact: true }));
        await absent(
          page.getByRole("button", { name: "Move to Business", exact: true }),
        );
        await capturePage(page, output, "review-loading-checking", width);
        const checked = page.waitForResponse(/\/billing$/);
        await control(origin, { pages: { held: [item.route] } });
        await (await checked).finished();
        await visible(page.getByText("Pricing this choice", { exact: true }));
      }
      if (item.name === "history-loading" && width !== 390)
        await visible(page.getByText("Finished", { exact: true }));
    },
  }));
}
