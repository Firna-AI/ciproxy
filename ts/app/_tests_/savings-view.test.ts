import assert from "node:assert/strict";
import test from "node:test";

import type { SavingsPage } from "../src/api/savings-types.ts";
import { costRows, savingsNotices } from "../src/view/savings-costs.ts";
import {
  costBars,
  priceListNote,
  savingsHero,
  savingsSubtitle,
  startsSeries,
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
    costRows(page).map((row) => [
      row.key,
      row.jobs,
      row.minutes,
      plain(row.cost),
    ]),
    [
      ["machines", "102", "498", "At least $105.42"],
      ["boat", "7", "41", "$10.00"],
      ["plan", "", "", "$47.74"],
      ["total", "109", "539", "Incomplete"],
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

test("the daily chart keeps own and Boat starts apart, and the note names each price list once in date order", () => {
  const page = report({
    ...quietDay,
    days: [
      {
        day: "2026-11-03",
        machines: { starts: "40", minutes: "190" },
        boat: { starts: "3", minutes: "18" },
      },
      {
        day: "2026-11-04",
        machines: { starts: "62", minutes: "308" },
        boat: { starts: "4", minutes: "23" },
      },
    ],
  });
  assert.deepEqual(startsSeries(page), {
    days: ["3", "4"],
    machines: [40, 62],
    boat: [3, 4],
  });
  const lists = (dates: [string, string]) =>
    plain(
      priceListNote(
        report({
          comparisons: quietDay.comparisons.map((item, index) => ({
            ...item,
            price_list_date: dates[index] ?? item.price_list_date,
          })),
        }),
      ),
    );
  assert.deepEqual(
    [lists(["2026-11-04", "2026-11-04"]), lists(["2026-11-11", "2026-11-04"])],
    [
      "Blacksmith and GitHub-hosted costs use their list prices per minute for the same VM sizes, from the price lists of 4 November 2026.",
      "Blacksmith and GitHub-hosted costs use their list prices per minute for the same VM sizes, from the price lists of 4 November 2026 and 11 November 2026.",
    ],
  );
});
