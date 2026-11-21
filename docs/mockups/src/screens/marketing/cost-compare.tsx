// The cost comparison of the landing page: one new Hetzner AX42 at its list
// price that runs jobs in every minute of a month, against the same job
// minutes on Blacksmith and on GitHub-hosted runners. Every number comes from
// fixtures/cost-claim.ts. A small caption names the basis and links to the
// calculation, a question of the pricing page.

import { TextLink } from "../../components/actions.tsx";
import { paths } from "../../components/paths.ts";
import { costClaim } from "../../fixtures/cost-claim.ts";
import {
  formatMoney,
  formatMoneyCents,
  unbroken,
} from "../../fixtures/format.ts";
import { planChoiceText } from "../../fixtures/plan-choice.ts";
import { listPrices } from "../../fixtures/price-lists.ts";
import { savingId } from "./saving-calculation.tsx";
import { Section, SectionHead } from "./section.tsx";
import "./cost.css";

type Bar = { label: string; value: number; accent: boolean };

const bars: readonly Bar[] = [
  { label: "With ciproxy", value: costClaim.withCiproxy, accent: true },
  ...costClaim.alternatives.map((alternative) => ({
    label: alternative.name,
    value: alternative.cost,
    accent: false,
  })),
];

const parts: readonly { label: string; value: number }[] = [
  { label: costClaim.machine, value: listPrices.machine.amount },
  {
    label: `ciproxy ${planChoiceText(costClaim.plan)}`,
    value: costClaim.plan.price,
  },
];

function CostBars() {
  const max = Math.max(...bars.map((bar) => bar.value));
  return (
    <figure
      className="cost-chart"
      aria-label="Cost of the same job minutes in a month"
    >
      {bars.map((bar) => (
        <div key={bar.label} className="cost-row">
          <span className={`cost-label${bar.accent ? " strong" : ""}`}>
            {bar.label}
          </span>
          <span className="cost-track">
            <span
              className={`cost-fill${bar.accent ? " is-accent" : ""}`}
              style={{ width: `calc((100% - 88px) * ${bar.value / max})` }}
            />
            <span className="cost-value num">{formatMoney(bar.value)}</span>
          </span>
        </div>
      ))}
    </figure>
  );
}

export function CostCompare() {
  return (
    <Section className="cost">
      <SectionHead
        eyebrow="Cost"
        title={`The same jobs for up to ${costClaim.upToPercent}% less`}
        lead={
          <>
            Your own machine against the same job minutes on Blacksmith and on{" "}
            <span className="nowrap">GitHub-hosted</span> runners.
          </>
        }
      />
      <div className="cost-panel">
        <div className="cost-main">
          <p className="cost-saved">
            <span className="cost-saved-value num">
              {formatMoney(costClaim.saved)}
            </span>
            <span className="cost-saved-text">
              a month less than {costClaim.cheaper.runners}
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
            <dd className="num">{formatMoneyCents(costClaim.withCiproxy)}</dd>
          </div>
        </dl>
      </div>
      <p className="caption cost-note">
        One new {costClaim.machine} at its list price runs jobs in every minute
        of a {costClaim.monthDays}-day month, at{" "}
        {unbroken(`${costClaim.averageJobMinutes} minutes`)} a job.{" "}
        <TextLink to={paths.pricing} fragment={savingId}>
          See the calculation
        </TextLink>
      </p>
    </Section>
  );
}
