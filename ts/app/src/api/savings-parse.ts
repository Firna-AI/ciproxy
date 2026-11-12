// Readers for the savings report. Every amount and count stays a decimal
// string, minutes keep their fraction, and a cost that the API leaves out
// stays null, so a screen never shows an unknown cost as zero.

import { readBannerField } from "./banner-reader.ts";
import { readVmSize } from "./jobs-parse.ts";
import {
  array,
  boolean,
  date,
  decimal,
  fraction,
  object,
  oneOf,
  optional,
  parser,
  ParseError,
  string,
} from "./parse-core.ts";
import type {
  SavingsAlternative,
  SavingsComparison,
  SavingsCostGap,
  SavingsDay,
  SavingsPage,
  SavingsPriceGap,
  SavingsPriceGapReason,
  SavingsRange,
  SavingsRuntime,
} from "./savings-types.ts";

export const savingsRange = oneOf<SavingsRange>({
  last30_days: true,
  this_month: true,
  last_month: true,
});

const alternative = oneOf<SavingsAlternative>({
  blacksmith: true,
  github: true,
});

const priceGapReason = oneOf<SavingsPriceGapReason>({
  unavailable: true,
  ambiguous: true,
});

function runtime(value: unknown): SavingsRuntime {
  const fields = object(value, "runtime");
  return { starts: decimal(fields.starts), minutes: fraction(fields.minutes) };
}

function day(value: unknown): SavingsDay {
  const fields = object(value, "report day");
  return {
    day: date(fields.day),
    machines: runtime(fields.machines),
    boat: runtime(fields.boat),
  };
}

function priceGap(value: unknown): SavingsPriceGap {
  const fields = object(value, "missing price");
  return {
    reason: priceGapReason(fields.reason),
    size: readVmSize(fields.size),
  };
}

function comparison(value: unknown): SavingsComparison {
  const fields = object(value, "comparison");
  return {
    alternative: alternative(fields.alternative),
    price_list_date: date(fields.price_list_date),
    cost_cents: optional(fields.cost_cents, decimal),
    savings_cents: optional(fields.savings_cents, decimal),
    missing_prices: array(fields.missing_prices, priceGap),
  };
}

function costGap(value: unknown): SavingsCostGap {
  const fields = object(value, "missing cost");
  switch (fields.kind) {
    case "workspace_start":
      return { kind: "workspace_start" };
    case "machine_registration":
      return {
        kind: "machine_registration",
        machine_id: string(fields.machine_id),
      };
    case "machine_price":
      return {
        kind: "machine_price",
        machine_id: string(fields.machine_id),
        day: date(fields.day),
      };
    case "plan_price":
      return { kind: "plan_price", day: date(fields.day) };
    default:
      throw new ParseError("missing cost");
  }
}

export const parseSavings = parser((value): SavingsPage => {
  const fields = object(value, "savings report");
  return {
    range: savingsRange(fields.range),
    from: date(fields.from),
    until: date(fields.until),
    machines: runtime(fields.machines),
    boat: runtime(fields.boat),
    days: array(fields.days, day),
    known_machine_cents: decimal(fields.known_machine_cents),
    boat_cents: decimal(fields.boat_cents),
    known_plan_cents: decimal(fields.known_plan_cents),
    total_cents: optional(fields.total_cents, decimal),
    missing_costs: array(fields.missing_costs, costGap),
    comparisons: array(fields.comparisons, comparison),
    lead: optional(fields.lead, alternative),
    can_add_machine_costs: boolean(fields.can_add_machine_costs),
    refresh_complete: boolean(fields.refresh_complete),
    banner: readBannerField(fields.banner),
  };
});
