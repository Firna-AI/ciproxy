// Starts by day, on own machines and on Boat. Outside a design direction it
// is the Firna bar chart. Inside one, it stretches to its panel: stacked bars,
// or thin lines over a faint area, with hairline grid lines and the axis
// labels in the monospace face. A visually hidden table gives screen readers
// the values, as the table view of the Firna chart does.

import type { Viewport } from "@mokly/mokly";
import { BarChart } from "@firna/ui/chart";

import { formatCount } from "../fixtures/format.ts";
import { useDirection, type ChartKind } from "../theme/direction.ts";
import { seriesColors } from "../theme/firna.tsx";
import "./starts-chart.css";

type StartsChartProps = {
  viewport: Viewport;
  labels: readonly string[];
  own: readonly number[];
  boat: readonly number[];
  caption: string;
};

export function StartsChart(props: StartsChartProps) {
  const direction = useDirection();
  if (direction) return <ScaledChart {...props} kind={direction.chart} />;
  const { viewport, labels, own, boat, caption } = props;
  return (
    <BarChart
      categories={labels}
      series={[
        {
          id: "own",
          label: "Own machines",
          data: own,
          color: seriesColors.own,
        },
        { id: "boat", label: "Boat", data: boat, color: seriesColors.boat },
      ]}
      mode="stacked"
      height={190}
      defaultWidth={viewport === "mobile" ? 300 : 380}
      caption={caption}
      showLegend
    />
  );
}

/** A round top for the value axis, with three or four steps. */
function axisOf(value: number): { max: number; ticks: number[] } {
  const rough = value / 3;
  const magnitude = 10 ** Math.floor(Math.log10(rough));
  const step =
    [1, 2, 2.5, 5, 10].map((m) => m * magnitude).find((s) => s >= rough) ??
    10 * magnitude;
  const max = Math.ceil(value / step) * step;
  const ticks = Array.from({ length: max / step + 1 }, (_, i) => i * step);
  return { max, ticks };
}

function linePath(values: readonly number[], max: number): string {
  return values
    .map((v, i) => `${i === 0 ? "M" : "L"}${i + 0.5} ${100 - (v / max) * 100}`)
    .join("");
}

function ScaledChart({
  labels,
  own,
  boat,
  caption,
  kind,
}: StartsChartProps & { kind: ChartKind }) {
  const totals = own.map((v, i) => v + (boat[i] ?? 0));
  const peak = kind === "bars" ? Math.max(...totals) : Math.max(...own);
  const { max, ticks } = axisOf(peak);
  const days = labels.length;
  const summary = `Starts by day, ${caption}: ${formatCount(own.reduce((a, b) => a + b, 0))} on own machines and ${formatCount(boat.reduce((a, b) => a + b, 0))} on Boat.`;
  return (
    <figure className={`sc sc-${kind}`} aria-label={summary}>
      <div className="sc-y" aria-hidden="true">
        {ticks.map((tick) => (
          <span key={tick} style={{ bottom: `${(tick / max) * 100}%` }}>
            {formatCount(tick)}
          </span>
        ))}
      </div>
      <div className="sc-plot">
        {ticks.map((tick) => (
          <span
            key={tick}
            className="sc-grid"
            style={{ bottom: `${(tick / max) * 100}%` }}
          />
        ))}
        <svg
          viewBox={`0 0 ${days} 100`}
          preserveAspectRatio="none"
          aria-hidden="true"
        >
          {kind === "bars"
            ? own.map((v, i) => {
                const ownHeight = (v / max) * 100;
                const boatHeight = ((boat[i] ?? 0) / max) * 100;
                return (
                  <g key={labels[i] ?? i}>
                    <rect
                      className="sc-own"
                      x={i + 0.2}
                      y={100 - ownHeight}
                      width={0.6}
                      height={ownHeight}
                    />
                    <rect
                      className="sc-boat"
                      x={i + 0.2}
                      y={100 - ownHeight - boatHeight}
                      width={0.6}
                      height={Math.max(boatHeight - 0.8, 0)}
                    />
                  </g>
                );
              })
            : [
                { key: "boat", values: boat },
                { key: "own", values: own },
              ].map((series) => (
                <g key={series.key} className={`sc-${series.key}`}>
                  <path
                    className="sc-area"
                    d={`${linePath(series.values, max)}L${days - 0.5} 100L0.5 100Z`}
                  />
                  <path
                    className="sc-line"
                    d={linePath(series.values, max)}
                    vectorEffect="non-scaling-stroke"
                  />
                </g>
              ))}
        </svg>
      </div>
      <div className="sc-x" aria-hidden="true">
        {labels.map((label, i) => (
          <span key={`${label}-${i}`}>{i % 3 === 0 ? label : ""}</span>
        ))}
      </div>
      <figcaption className="sc-foot">
        <span className="sc-legend">
          <span className="sc-key sc-own">Own machines</span>
          <span className="sc-key sc-boat">Boat</span>
        </span>
        <span className="caption">{caption}</span>
      </figcaption>
      <div className="sc-table">
        <table>
          <thead>
            <tr>
              <th scope="col">Day</th>
              <th scope="col">Own machines</th>
              <th scope="col">Boat</th>
            </tr>
          </thead>
          <tbody>
            {labels.map((label, i) => (
              <tr key={`${label}-${i}`}>
                <th scope="row">{label}</th>
                <td>{formatCount(own[i] ?? 0)}</td>
                <td>{formatCount(boat[i] ?? 0)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </figure>
  );
}
