// Starts by day, on own machines and on Boat: stacked bars of one value axis
// from the Firna chart family, in the two series colours of the chart tokens.
// The legend names both series, and the chart keeps its table view, so no
// value lives in colour alone.

import { BarChart } from "@firna/ui/chart";

import { chartColors } from "@ciproxy/tokens";

import { Panel } from "../../components/panel.tsx";
import type { SavingsPage } from "../../api/savings-types.ts";
import { formatCount, formatUtcRange } from "../../view/exact-format.ts";
import { startsSeries } from "../../view/savings-report.ts";

export function StartsPanel({ page }: { page: SavingsPage }) {
  const series = startsSeries(page);
  const period = formatUtcRange(page.from, page.until);
  const summary = `Starts by day, ${period}: ${formatCount(page.machines.starts)} on own machines and ${formatCount(page.boat.starts)} on Boat.`;
  return (
    <Panel title="Starts by day">
      <BarChart
        mode="stacked"
        categories={series.days}
        series={[
          {
            id: "machines",
            label: "Own machines",
            data: series.machines,
            color: chartColors.own,
          },
          {
            id: "boat",
            label: "Boat",
            data: series.boat,
            color: chartColors.boat,
          },
        ]}
        height={220}
        defaultWidth={480}
        caption={period}
        valueFormat={(value) => formatCount(BigInt(Math.round(value)))}
        accessibilityLabel={summary}
      />
    </Panel>
  );
}
