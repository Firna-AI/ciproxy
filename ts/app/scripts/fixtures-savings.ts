import type { components } from "../../api/index.ts";
import { fixtureTime } from "./fixtures-core.ts";
import { fixtureMachineId, fixtureVmSize } from "./fixtures-jobs.ts";

type SavingsPage = components["schemas"]["SavingsPage"];
type SavingsRange = components["schemas"]["SavingsRange"];

export const savingsModes = [
  "report",
  "missing-costs",
  "negative",
  "missing-price",
  "ambiguous-price",
  "empty",
  "refresh-incomplete",
] as const;
export type SavingsMode = (typeof savingsModes)[number];

export const savingsRanges = [
  "last30_days",
  "this_month",
  "last_month",
] as const;

const periods = {
  last30_days: {
    from: "2026-10-08",
    until: "2026-11-07",
    machine: "5845",
    plan: "4774",
    boat: "1000",
    blacksmith: "18000",
    github: "26000",
  },
  this_month: {
    from: "2026-11-01",
    until: "2026-11-07",
    machine: "1200",
    plan: "980",
    boat: "600",
    blacksmith: "5000",
    github: "7000",
  },
  last_month: {
    from: "2026-10-01",
    until: "2026-11-01",
    machine: "6000",
    plan: "4900",
    boat: "2000",
    blacksmith: "20000",
    github: "28000",
  },
} satisfies Record<
  SavingsRange,
  {
    from: string;
    until: string;
    machine: string;
    plan: string;
    boat: string;
    blacksmith: string;
    github: string;
  }
>;

function reportDays(
  range: SavingsRange,
  empty: boolean,
): components["schemas"]["SavingsDay"][] {
  const { from, until } = periods[range];
  const count = (Date.parse(until) - Date.parse(from)) / 86_400_000;
  return Array.from({ length: count }, (_, index) => {
    const time = Date.parse(from) + index * 86_400_000;
    const seed = Math.floor(
      (time - Date.parse(fixtureTime.slice(0, 10))) / 86_400_000,
    );
    const starts = empty ? 0 : 200 + ((seed + 90) % 7) * 20;
    const boat = empty ? 0 : 20 + ((seed + 90) % 3) * 10;
    return {
      day: new Date(time).toISOString().slice(0, 10),
      machines: { starts: String(starts), minutes: String(starts * 4) },
      boat: { starts: String(boat), minutes: String(boat * 6) },
    } satisfies components["schemas"]["SavingsDay"];
  });
}

function runtime(
  days: components["schemas"]["SavingsDay"][],
  place: "machines" | "boat",
): components["schemas"]["SavingsRuntime"] {
  return {
    starts: String(
      days.reduce((sum, day) => sum + BigInt(day[place].starts), 0n),
    ),
    minutes: String(
      days.reduce((sum, day) => sum + BigInt(day[place].minutes), 0n),
    ),
  };
}

export function savingsReport(
  mode: SavingsMode,
  range: SavingsRange,
  canAddCosts: boolean,
  banner: components["schemas"]["PageBanner"] | null,
): SavingsPage {
  const period = periods[range];
  const empty = mode === "empty";
  const missingCosts = mode === "missing-costs";
  const missingPrice = mode === "missing-price" || mode === "ambiguous-price";
  const machine =
    empty || missingCosts
      ? "0"
      : mode === "negative"
        ? "30000"
        : period.machine;
  const plan = empty ? "0" : period.plan;
  const boat = empty ? "0" : period.boat;
  const total = missingCosts
    ? null
    : String(BigInt(machine) + BigInt(plan) + BigInt(boat));
  const days = reportDays(range, empty);
  return {
    banner,
    range,
    from: period.from,
    until: period.until,
    can_add_machine_costs: canAddCosts,
    refresh_complete: mode !== "refresh-incomplete",
    days,
    machines: runtime(days, "machines"),
    boat: runtime(days, "boat"),
    boat_cents: boat,
    known_machine_cents: machine,
    known_plan_cents: plan,
    total_cents: total,
    missing_costs: missingCosts
      ? [
          {
            kind: "machine_price",
            machine_id: fixtureMachineId,
            day: period.from,
          },
        ]
      : [],
    lead: missingPrice ? null : "blacksmith",
    comparisons: (["blacksmith", "github"] as const).map((alternative) => {
      const gap = missingPrice && alternative === "blacksmith";
      const cost = gap ? null : empty ? "0" : period[alternative];
      return {
        alternative,
        price_list_date: "2026-11-01",
        cost_cents: cost,
        savings_cents:
          cost === null || total === null
            ? null
            : String(BigInt(cost) - BigInt(total)),
        missing_prices: gap
          ? [
              {
                size: fixtureVmSize,
                reason:
                  mode === "ambiguous-price" ? "ambiguous" : "unavailable",
              },
            ]
          : [],
      } satisfies components["schemas"]["SavingsComparison"];
    }),
  } satisfies SavingsPage;
}
