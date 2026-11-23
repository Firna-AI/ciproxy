// The parts of the ciproxy cost: own machines, a row for the overflow of each
// provider that ran jobs in the range, and the plan. A money column, so
// every amount shows cents. A cost that is known only in part shows its
// known part as "At least", a machine cost that nobody set says Not set, and
// the total of an incomplete report says Incomplete.

import { DataTable, type Column } from "../../components/data-table.tsx";
import { formatCount, formatMoneyCents } from "../../fixtures/format.ts";
import { providerName } from "../../fixtures/providers.ts";
import {
  jobCount,
  totalCost,
  type ReportFigures,
} from "../../fixtures/savings-states.ts";

type CostRow = {
  key: string;
  label: string;
  jobs: string;
  minutes: string;
  cost: string;
  total?: boolean;
};

function partCost(cost: number | null, partial: boolean | undefined): string {
  if (cost === null) return "Not set";
  return partial
    ? `At least ${formatMoneyCents(cost)}`
    : formatMoneyCents(cost);
}

function rows(figures: ReportFigures): CostRow[] {
  const total = totalCost(figures);
  const minutes = figures.providers.reduce(
    (sum, entry) => sum + entry.minutes,
    figures.own.minutes,
  );
  return [
    {
      key: "own",
      label: figures.ownLabel,
      jobs: formatCount(figures.own.starts),
      minutes: formatCount(figures.own.minutes),
      cost: partCost(figures.ownCost, figures.ownPartial),
    },
    ...figures.providers.map((entry) => ({
      key: entry.provider,
      label: `${providerName(entry.provider)} overflow`,
      jobs: formatCount(entry.starts),
      minutes: formatCount(entry.minutes),
      cost: formatMoneyCents(entry.cost),
    })),
    {
      key: "plan",
      label: figures.planLabel,
      jobs: "",
      minutes: "",
      cost: partCost(figures.planCost, figures.planPartial),
    },
    {
      key: "total",
      label: "Total",
      jobs: formatCount(jobCount(figures)),
      minutes: formatCount(minutes),
      cost: total === null ? "Incomplete" : formatMoneyCents(total),
      total: true,
    },
  ];
}

function cellValue(row: CostRow, key: string): string {
  switch (key) {
    case "jobs":
      return row.jobs;
    case "minutes":
      return row.minutes;
    case "cost":
      return row.cost;
    default:
      return row.label;
  }
}

const columns: readonly Column[] = [
  { key: "label", label: "", flex: 3 },
  { key: "jobs", label: "Jobs", flex: 2, align: "right" },
  { key: "minutes", label: "Minutes", flex: 2, align: "right" },
  { key: "cost", label: "Cost", flex: 2, align: "right" },
];

export function CostTable({ figures }: { figures: ReportFigures }) {
  return (
    <DataTable
      columns={columns}
      rows={rows(figures)}
      rowKey={(row) => row.key}
      cell={(row, key) => (
        <span className={row.total ? "cost-total" : undefined}>
          {cellValue(row, key)}
        </span>
      )}
    />
  );
}
