// What the same jobs cost with ciproxy, on Blacksmith, and on GitHub-hosted
// runners. Emphasis form: the ciproxy bar in the accent, the others in grey,
// each bar labelled with its name and value. A cost that is not known has no
// bar, so no bar stands for an unknown amount.

import { formatMoney } from "../../fixtures/format.ts";
import {
  totalCost,
  type ReportFigures,
} from "../../fixtures/savings-states.ts";

type Bar = { key: string; label: string; value: number; accent: boolean };

function barsOf(figures: ReportFigures): Bar[] {
  const candidates = [
    {
      key: "ciproxy",
      label: "With ciproxy",
      value: totalCost(figures),
      accent: true,
    },
    {
      key: "blacksmith",
      label: "Blacksmith",
      value: figures.alternatives.blacksmith,
      accent: false,
    },
    {
      key: "github",
      label: "GitHub-hosted",
      value: figures.alternatives.github,
      accent: false,
    },
  ];
  return candidates.flatMap((bar) =>
    bar.value === null ? [] : [{ ...bar, value: bar.value }],
  );
}

export function CostBars({ figures }: { figures: ReportFigures }) {
  const bars = barsOf(figures);
  const max = Math.max(...bars.map((bar) => bar.value));
  return (
    <figure className="cost-bars" aria-label="Cost of the same jobs">
      {bars.map((bar) => (
        <div
          key={bar.key}
          className="cost-bar-row"
          title={`${bar.label}: ${formatMoney(bar.value)}`}
        >
          <span className={`cost-bar-label${bar.accent ? " strong" : ""}`}>
            {bar.label}
          </span>
          <span className="cost-bar-track">
            <span
              className={`cost-bar${bar.accent ? " is-accent" : ""}`}
              style={{ width: `calc((100% - 80px) * ${bar.value / max})` }}
            />
            <span className="cost-bar-value num">{formatMoney(bar.value)}</span>
          </span>
        </div>
      ))}
    </figure>
  );
}
