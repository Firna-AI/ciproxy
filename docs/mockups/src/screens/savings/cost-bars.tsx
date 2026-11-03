// What the same jobs cost with ciproxy, on Blacksmith, and on GitHub-hosted
// runners. Emphasis form: the ciproxy bar in the accent, the others in grey,
// each bar labelled with its name and value.

import { formatMoney } from "../../fixtures/format.ts";
import { savings, yourCost } from "../../fixtures/savings.ts";

type Bar = { key: string; label: string; value: number; accent: boolean };

const bars: readonly Bar[] = [
  { key: "ciproxy", label: "With ciproxy", value: yourCost, accent: true },
  {
    key: "blacksmith",
    label: "Blacksmith",
    value: savings.alternatives.blacksmith,
    accent: false,
  },
  {
    key: "github",
    label: "GitHub-hosted",
    value: savings.alternatives.github,
    accent: false,
  },
];

export function CostBars({ withCiproxy = true }: { withCiproxy?: boolean }) {
  const shown = withCiproxy ? bars : bars.filter((bar) => !bar.accent);
  const max = Math.max(...bars.map((bar) => bar.value));
  return (
    <figure
      className="cost-bars"
      aria-label="Cost of the same jobs, last 30 days"
    >
      {shown.map((bar) => (
        <div
          key={bar.key}
          className="cost-bar-row"
          title={`${bar.label}: ${formatMoney(Math.round(bar.value))}`}
        >
          <span className={`cost-bar-label${bar.accent ? " strong" : ""}`}>
            {bar.label}
          </span>
          <span className="cost-bar-track">
            <span
              className={`cost-bar${bar.accent ? " is-accent" : ""}`}
              style={{ width: `calc((100% - 72px) * ${bar.value / max})` }}
            />
            <span className="cost-bar-value num">
              {formatMoney(Math.round(bar.value))}
            </span>
          </span>
        </div>
      ))}
    </figure>
  );
}
