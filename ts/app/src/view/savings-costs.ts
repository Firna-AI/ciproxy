// The cost table and the notices of the savings report. A cost that the API
// knows only in part shows its known part as "At least", a machine cost that
// nobody set says "Not set", and a plan price that ciproxy lacks says
// "Unknown". The total of an incomplete report says "Incomplete". Totals of
// starts and minutes add both places exactly.

import type {
  SavingsComparison,
  SavingsCostGap,
  SavingsPage,
} from "../api/savings-types.ts";
import type { NoticeTone } from "./page-model.ts";
import {
  centsSign,
  formatCents,
  formatCount,
  formatWhole,
  sumDecimals,
} from "./exact-format.ts";
import { formatGigabytes, joinWords } from "./format.ts";
import { totalStarts } from "./savings-report.ts";

export type CostRow = {
  key: string;
  label: string;
  jobs: string;
  minutes: string;
  cost: string;
  total: boolean;
};

const machineGaps = new Set<SavingsCostGap["kind"]>([
  "machine_price",
  "machine_registration",
]);

function gapped(page: SavingsPage, kinds: ReadonlySet<string>): boolean {
  return page.missing_costs.some((gap) => kinds.has(gap.kind));
}

function partCost(known: string, gap: boolean, none: string): string {
  if (!gap) return formatCents(known, "column");
  if (centsSign(known) === 0) return none;
  return `At least ${formatCents(known, "column")}`;
}

export function costRows(page: SavingsPage): CostRow[] {
  const machineGap = gapped(page, machineGaps);
  const priceMissing = page.missing_costs.some(
    (gap) => gap.kind === "machine_price",
  );
  const planGap = gapped(page, new Set(["plan_price", "workspace_start"]));
  const total = page.total_cents ?? null;
  return [
    {
      key: "machines",
      label: "Own machines",
      jobs: formatCount(page.machines.starts),
      minutes: formatWhole(page.machines.minutes),
      cost: partCost(
        page.known_machine_cents,
        machineGap,
        priceMissing ? "Not set" : "Unknown",
      ),
      total: false,
    },
    {
      key: "boat",
      label: "Boat overflow",
      jobs: formatCount(page.boat.starts),
      minutes: formatWhole(page.boat.minutes),
      cost: formatCents(page.boat_cents, "column"),
      total: false,
    },
    {
      key: "plan",
      label: "ciproxy plan",
      jobs: "",
      minutes: "",
      cost: partCost(page.known_plan_cents, planGap, "Unknown"),
      total: false,
    },
    {
      key: "total",
      label: "Total",
      jobs: formatCount(totalStarts(page)),
      minutes: formatWhole(
        sumDecimals([page.machines.minutes, page.boat.minutes]),
      ),
      cost: total === null ? "Incomplete" : formatCents(total, "column"),
      total: true,
    },
  ];
}

export type SavingsNotice = {
  key: string;
  tone: NoticeTone;
  title: string;
  body: string;
  /** The notice offers to add machine costs, to a caller who may. */
  addCosts: boolean;
};

const priceNames = { blacksmith: "Blacksmith", github: "GitHub" } as const;

function priceNotice(item: SavingsComparison): SavingsNotice {
  const sizes = item.missing_prices.map(
    (gap) =>
      `${gap.size.vcpus} vCPU and ${formatGigabytes(gap.size.memory_bytes)}`,
  );
  if (sizes.length === 0) {
    return {
      key: `price-${item.alternative}`,
      tone: "info",
      title: `${priceNames[item.alternative]} has no complete price`,
      body: "The report leaves out its cost and the savings figure.",
      addCosts: false,
    };
  }
  const count = sizes.length === 1 ? "one VM size" : `${sizes.length} VM sizes`;
  const ambiguous = item.missing_prices.every(
    (gap) => gap.reason === "ambiguous",
  );
  const which = joinWords(sizes, "and");
  return {
    key: `price-${item.alternative}`,
    tone: "info",
    title: `${priceNames[item.alternative]} has no price for ${count}`,
    body: ambiguous
      ? `Its price list gives more than one price for ${which}, so the report leaves out its cost and the savings figure.`
      : `Its price list has no size with at least ${which}, so the report leaves out its cost and the savings figure.`,
    addCosts: false,
  };
}

export function savingsNotices(page: SavingsPage): SavingsNotice[] {
  const notices: SavingsNotice[] = [];
  if (page.missing_costs.some((gap) => gap.kind === "machine_price")) {
    notices.push({
      key: "machine-costs",
      tone: "warning",
      title: "Add what your machines cost",
      body: "The report needs the monthly cost of each machine to show your savings.",
      addCosts: true,
    });
  }
  if (page.missing_costs.some((gap) => gap.kind !== "machine_price")) {
    notices.push({
      key: "unknown-costs",
      tone: "info",
      title: "Some costs are not known",
      body: "Some plan prices or machine registration dates are not known, so the report shows no savings. It still shows the costs that ciproxy knows.",
      addCosts: false,
    });
  }
  for (const item of page.comparisons) {
    if ((item.cost_cents ?? null) === null) notices.push(priceNotice(item));
  }
  return notices;
}
