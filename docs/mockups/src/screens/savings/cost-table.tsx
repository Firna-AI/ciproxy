// The parts of the ciproxy cost: own machines, Boat overflow, and the plan. A
// money column, so every amount shows cents.

import { DataTable, type Column } from "../../components/data-table.tsx";
import { formatCount, formatMoneyCents } from "../../fixtures/format.ts";
import { savings, yourCost } from "../../fixtures/savings.ts";
import { ownMachines } from "../../fixtures/tiers.ts";

type CostRow = {
  key: string;
  label: string;
  jobs: string;
  minutes: string;
  cost: string;
  total?: boolean;
};

function rows(missing: boolean): CostRow[] {
  const { starts, minutes, cost } = savings;
  return [
    {
      key: "own",
      label: ownMachines(3),
      jobs: formatCount(starts.own),
      minutes: formatCount(minutes.own),
      cost: missing ? "Not set" : formatMoneyCents(cost.own),
    },
    {
      key: "boat",
      label: "Boat overflow",
      jobs: formatCount(starts.boat),
      minutes: formatCount(minutes.boat),
      cost: formatMoneyCents(cost.boat),
    },
    {
      key: "plan",
      label: "ciproxy Team",
      jobs: "",
      minutes: "",
      cost: formatMoneyCents(cost.plan),
    },
    {
      key: "total",
      label: "Total",
      jobs: formatCount(starts.own + starts.boat),
      minutes: formatCount(minutes.own + minutes.boat),
      cost: missing ? "Incomplete" : formatMoneyCents(yourCost),
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

export function CostTable({ missing }: { missing: boolean }) {
  return (
    <DataTable
      columns={columns}
      rows={rows(missing)}
      rowKey={(row) => row.key}
      cell={(row, key) => (
        <span className={row.total ? "cost-total" : undefined}>
          {cellValue(row, key)}
        </span>
      )}
    />
  );
}
