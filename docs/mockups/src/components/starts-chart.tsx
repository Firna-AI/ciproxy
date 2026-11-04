// Starts by day, on own machines and on Boat: stacked bars that stretch to
// their panel, hairline grid lines, and the axis labels in the monospace face.
// A visually hidden table gives screen readers the values.

import { formatCount } from "../fixtures/format.ts";
import "./starts-chart.css";

type StartsChartProps = {
  labels: readonly string[];
  own: readonly number[];
  boat: readonly number[];
  caption: string;
};

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

export function StartsChart({ labels, own, boat, caption }: StartsChartProps) {
  const totals = own.map((v, i) => v + (boat[i] ?? 0));
  const { max, ticks } = axisOf(Math.max(...totals));
  const days = labels.length;
  const summary = `Starts by day, ${caption}: ${formatCount(own.reduce((a, b) => a + b, 0))} on own machines and ${formatCount(boat.reduce((a, b) => a + b, 0))} on Boat.`;
  return (
    <figure className="sc" aria-label={summary}>
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
          {own.map((v, i) => {
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
          })}
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
