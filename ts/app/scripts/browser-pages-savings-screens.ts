import { savingsHref } from "../src/view/savings-report.ts";
import type { Page } from "./browser-driver.ts";
import { insideRegion, separateRegions } from "./browser-pages-helpers.ts";
import { workspaceId } from "./fixtures-core.ts";
import { screenRuns, type PageScreen } from "./browser-pages-screens.ts";

export async function visibleCostRows(
  page: Page,
  expected = ["$58.45", "$10.00", "$47.74"],
): Promise<void> {
  const table = page.getByRole("region", {
    name: "Your cost with ciproxy",
    exact: true,
  });
  for (const [index, label] of [
    "Own machines",
    "Boat overflow",
    "ciproxy plan",
  ].entries()) {
    const cell = table.getByText(label, { exact: true });
    await insideRegion(table, cell);
    const amount = expected[index];
    if (amount !== undefined)
      await insideRegion(table, table.getByText(amount, { exact: true }));
  }
  await separateRegions(
    table,
    page.getByRole("region", { name: "Starts by day", exact: true }),
  );
}

export function savingsScreenRuns(origin: string) {
  const path = savingsHref(workspaceId, "last30_days");
  const report: Pick<PageScreen, "heading" | "check"> = {
    heading: "Your cost with ciproxy",
    check: visibleCostRows,
  };
  const screens: PageScreen[] = [
    { name: "savings-report", scenario: "m21-savings-report", path, ...report },
    {
      name: "savings-missing-costs",
      scenario: "m21-savings-missing-costs",
      path,
      text: "Add what your machines cost",
      check: (page) => visibleCostRows(page, ["Not set", "$10.00", "$47.74"]),
    },
    {
      name: "savings-negative",
      scenario: "m21-savings-negative",
      path,
      text: /more than Blacksmith/,
      check: (page) => visibleCostRows(page, ["$300.00", "$10.00", "$47.74"]),
    },
    {
      name: "savings-missing-price",
      scenario: "m21-savings-missing-price",
      path,
      text: "Blacksmith has no price for one VM size",
      check: visibleCostRows,
    },
    {
      name: "savings-ambiguous-price",
      scenario: "m21-savings-ambiguous-price",
      path,
      text: /Its price list gives more than one price/,
      check: visibleCostRows,
    },
    {
      name: "savings-this-month",
      scenario: "m21-savings-report",
      path: savingsHref(workspaceId, "this_month"),
      heading: "Your cost with ciproxy",
      check: (page) => visibleCostRows(page, ["$12.00", "$6.00", "$9.80"]),
    },
    {
      name: "savings-last-month",
      scenario: "m21-savings-report",
      path: savingsHref(workspaceId, "last_month"),
      heading: "Your cost with ciproxy",
      check: (page) => visibleCostRows(page, ["$60.00", "$20.00", "$49.00"]),
    },
    {
      name: "savings-empty",
      scenario: "m21-savings-empty",
      path,
      heading: "Your savings report starts with your first job",
    },
    {
      name: "savings-empty-month",
      scenario: "m21-savings-empty",
      path: savingsHref(workspaceId, "this_month"),
      heading: "No jobs ran this month",
    },
    {
      name: "savings-refreshing",
      scenario: "m21-savings-refresh-incomplete",
      path,
      text: "Updating the report…",
      check: visibleCostRows,
    },
    {
      name: "savings-error",
      scenario: "m21-savings-error",
      path,
      heading: "Unable to load the savings report",
      statuses: [503],
    },
    {
      name: "savings-viewer",
      scenario: "m21-savings-missing-costs",
      pages: { role: "viewer" },
      path,
      text: "Ask an Admin",
      check: (page) => visibleCostRows(page, ["Not set", "$10.00", "$47.74"]),
    },
    {
      name: "savings-billing",
      scenario: "m21-savings-missing-costs",
      pages: { role: "billing" },
      path,
      text: "Ask an Admin",
      check: (page) => visibleCostRows(page, ["Not set", "$10.00", "$47.74"]),
    },
  ];
  return screenRuns(origin, screens);
}
