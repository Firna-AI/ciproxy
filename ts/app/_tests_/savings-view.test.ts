import assert from "node:assert/strict";
import test from "node:test";

import type { SavingsPage } from "../src/api/savings-types.ts";
import { costRows, savingsNotices } from "../src/view/savings-costs.ts";
import {
  costBars,
  savingsHero,
  savingsSubtitle,
} from "../src/view/savings-report.ts";
import { savingsReport } from "../scripts/fixtures-savings.ts";

/** The words of a value. Formats keep a value on one line with no-break spaces. */
function plain(value: string): string {
  return value.replaceAll("\u00a0", " ");
}

function report(overrides: Partial<SavingsPage>): SavingsPage {
  return {
    ...savingsReport("report", "last30_days", true, null),
    ...overrides,
  };
}

const quietDay = {
  machines: { starts: "102", minutes: "498" },
  boat: { starts: "7", minutes: "41" },
  total_cents: "713",
  lead: "blacksmith",
  comparisons: [
    {
      alternative: "blacksmith",
      price_list_date: "2026-11-04",
      cost_cents: "586",
      savings_cents: "-127",
      missing_prices: [],
    },
    {
      alternative: "github",
      price_list_date: "2026-11-04",
      cost_cents: "862",
      savings_cents: "149",
      missing_prices: [],
    },
  ],
} satisfies Partial<SavingsPage>;

test("a negative saving says more than, and keeps its amount and the other sign", () => {
  assert.deepEqual(savingsHero(report(quietDay)), {
    figure: "$1.27",
    text: "more than Blacksmith for the same 109 jobs, and $1.49 less than GitHub-hosted runners.",
  });
});

test("a missing machine cost hides the savings figure but keeps the known costs", () => {
  const page = report({
    ...quietDay,
    total_cents: null,
    known_machine_cents: "10542",
    missing_costs: [
      { kind: "machine_price", machine_id: "m-1", day: "2026-11-01" },
    ],
  });
  assert.equal(savingsHero(page), null);
  assert.deepEqual(
    costRows(page).map((row) => [row.key, plain(row.cost)]),
    [
      ["machines", "At least $105.42"],
      ["boat", "$10.00"],
      ["plan", "$47.74"],
      ["total", "Incomplete"],
    ],
  );
  assert.deepEqual(
    costBars(page).map((bar) => bar.key),
    ["blacksmith", "github"],
  );
  assert.deepEqual(
    savingsNotices(page).map((notice) => [notice.key, notice.addCosts]),
    [["machine-costs", true]],
  );
});

test("an alternative without a complete price shows no lead and names the size", () => {
  const page = report({
    lead: null,
    comparisons: [
      {
        alternative: "blacksmith",
        price_list_date: "2026-11-04",
        cost_cents: null,
        savings_cents: null,
        missing_prices: [
          {
            reason: "unavailable",
            size: {
              vcpus: 16,
              memory_bytes: "64000000000",
              disk_bytes: "251000000000",
            },
          },
        ],
      },
      ...quietDay.comparisons.slice(1),
    ],
  });
  assert.equal(savingsHero(page), null);
  assert.deepEqual(
    savingsNotices(page).map((notice) => [notice.title, plain(notice.body)]),
    [
      [
        "Blacksmith has no price for one VM size",
        "Its price list has no size with at least 16 vCPU and 64 GB, so the report leaves out its cost and the savings figure.",
      ],
    ],
  );
});

test("the report names its range with the last included UTC day", () => {
  const page = report({ from: "2026-10-20", until: "2026-11-19" });
  assert.equal(
    plain(savingsSubtitle(page)),
    "Last 30 days · 20 October – 18 November",
  );
  const oneDay = report({
    range: "this_month",
    from: "2026-11-01",
    until: "2026-11-02",
  });
  assert.equal(plain(savingsSubtitle(oneDay)), "This month · 1 November");
});
