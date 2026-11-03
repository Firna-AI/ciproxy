// The parts of the ciproxy cost: own machines, Boat overflow, and the plan.

import { TableCell } from "@firna/ui/table";

import { DataTable } from "../../components/data-table.tsx";
import { formatCount, formatMoneyCents } from "../../fixtures/format.ts";
import { savings, yourCost } from "../../fixtures/savings.ts";

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
      label: "3 own machines",
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

const columns = [
  { key: "label", label: "", flex: 3 },
  { key: "jobs", label: "Jobs", flex: 2, align: "right" as const },
  { key: "minutes", label: "Minutes", flex: 2, align: "right" as const },
  { key: "cost", label: "Cost", flex: 2, align: "right" as const },
];

export function CostTable({ missing }: { missing: boolean }) {
  return (
    <DataTable
      columns={columns}
      rows={rows(missing)}
      rowKey={(row) => row.key}
      size="sm"
      cell={(row, key) => {
        const value = cellValue(row, key);
        const numeric = key !== "label";
        return (
          <TableCell bold={row.total} align={numeric ? "right" : "left"}>
            <span className={numeric ? "num" : undefined}>{value}</span>
          </TableCell>
        );
      }}
    />
  );
}
