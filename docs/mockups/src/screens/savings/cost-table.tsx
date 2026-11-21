// The parts of the ciproxy cost: own machines, Boat overflow, and the plan. A
// money column, so every amount shows cents. A cost that is known only in
// part shows its known part as "At least", a machine cost that nobody set
// says Not set, and the total of an incomplete report says Incomplete.

import { DataTable, type Column } from "../../components/data-table.tsx";
import { formatCount, formatMoneyCents } from "../../fixtures/format.ts";
import {
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
  const { starts, minutes } = figures;
  const total = totalCost(figures);
  return [
    {
      key: "own",
      label: figures.ownLabel,
      jobs: formatCount(starts.own),
      minutes: formatCount(minutes.own),
      cost: partCost(figures.ownCost, figures.ownPartial),
    },
    {
      key: "boat",
      label: "Boat overflow",
      jobs: formatCount(starts.boat),
      minutes: formatCount(minutes.boat),
      cost: formatMoneyCents(figures.boatCost),
    },
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
      jobs: formatCount(starts.own + starts.boat),
      minutes: formatCount(minutes.own + minutes.boat),
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
