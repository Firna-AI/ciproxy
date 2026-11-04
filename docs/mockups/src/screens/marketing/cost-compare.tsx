// The cost comparison: one real month of the Linux CI of mokly-ai, with
// ciproxy against the same jobs on Blacksmith and on GitHub-hosted runners,
// and what the ciproxy total is made of. The values are the savings report's.

import {
  formatCount,
  formatMoney,
  formatMoneyCents,
} from "../../fixtures/format.ts";
import { ax42 } from "../../fixtures/machines.ts";
import { savings, yourCost } from "../../fixtures/savings.ts";
import { reference } from "../../fixtures/site.ts";
import { ownMachines, tiers } from "../../fixtures/tiers.ts";
import { Section, SectionHead } from "./section.tsx";
import "./cost.css";

type Bar = { label: string; value: number; accent: boolean };

const bars: readonly Bar[] = [
  { label: "With ciproxy", value: yourCost, accent: true },
  {
    label: "Blacksmith",
    value: savings.alternatives.blacksmith,
    accent: false,
  },
  { label: "GitHub-hosted", value: savings.alternatives.github, accent: false },
];

const parts: readonly { label: string; value: number }[] = [
  { label: ownMachines(3), value: savings.cost.own },
  { label: "Boat overflow", value: savings.cost.boat },
  { label: `ciproxy ${tiers.team.name}`, value: savings.cost.plan },
];

function CostBars() {
  const max = Math.max(...bars.map((bar) => bar.value));
  return (
    <figure
      className="cost-chart"
      aria-label="Cost of the same jobs in 30 days"
    >
      {bars.map((bar) => (
        <div key={bar.label} className="cost-row">
          <span className={`cost-label${bar.accent ? " strong" : ""}`}>
            {bar.label}
          </span>
          <span className="cost-track">
            <span
              className={`cost-fill${bar.accent ? " is-accent" : ""}`}
              style={{ width: `calc((100% - 72px) * ${bar.value / max})` }}
            />
            <span className="cost-value num">{formatMoney(bar.value)}</span>
          </span>
        </div>
      ))}
    </figure>
  );
}

export function CostCompare() {
  const saved = savings.alternatives.blacksmith - yourCost;
  return (
    <Section className="cost">
      <SectionHead
        eyebrow="Cost"
        title="The same jobs for less than half the price"
        lead={`The Linux CI of ${reference.org} in 30 days: ${formatCount(reference.jobs)} jobs, with up to ${reference.peakJobs} at once.`}
      />
      <div className="cost-panel">
        <div className="cost-main">
          <p className="cost-saved">
            <span className="cost-saved-value num">{formatMoney(saved)}</span>
            <span className="cost-saved-text">
              a month less than Blacksmith
            </span>
          </p>
          <CostBars />
        </div>
        <dl className="cost-parts">
          {parts.map((part) => (
            <div key={part.label} className="cost-part">
              <dt>{part.label}</dt>
              <dd className="num">{formatMoneyCents(part.value)}</dd>
            </div>
          ))}
          <div className="cost-part is-total">
            <dt>Total with ciproxy</dt>
            <dd className="num">{formatMoneyCents(yourCost)}</dd>
          </div>
        </dl>
      </div>
      <p className="caption cost-note">
        Blacksmith and GitHub-hosted costs use their list prices per minute for
        the same VM sizes. The machines are three Hetzner AX42 at{" "}
        {formatMoney(ax42.monthlyCost)} a month each.
      </p>
    </Section>
  );
}
