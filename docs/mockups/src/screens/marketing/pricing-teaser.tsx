// The pricing teaser of the landing page: the three tiers with their price and
// limits, and the way to the pricing page.

import { ArrowRight } from "lucide-react";

import { LinkAction } from "../../components/actions.tsx";
import { paths } from "../../components/paths.ts";
import { formatMoney } from "../../fixtures/format.ts";
import {
  ownMachines,
  startsText,
  tierOrder,
  tiers,
} from "../../fixtures/tiers.ts";
import { Actions, Cells, Section, SectionHead } from "./section.tsx";

export function PricingTeaser() {
  return (
    <Section flush>
      <SectionHead
        eyebrow="Pricing"
        title={["Start free.", "Upgrade when your CI grows."]}
        lead="Plans count the jobs that ciproxy starts. Every limit is soft, so your jobs keep running past it."
      />
      <Cells columns={3}>
        {tierOrder.map((key) => {
          const tier = tiers[key];
          return (
            <div key={key} className="mk-cell teaser">
              <h3 className="teaser-name">{tier.name}</h3>
              <p className="teaser-price">
                <span className="teaser-amount num">
                  {formatMoney(tier.price)}
                </span>
                <span className="muted">a month</span>
              </p>
              <p className="small teaser-limits">
                {startsText(tier.starts)} a month
                <br />
                {ownMachines(tier.machines)}
              </p>
            </div>
          );
        })}
      </Cells>
      <div className="mk-foot">
        <Actions>
          <LinkAction to={paths.signIn} tone="primary">
            Start free
          </LinkAction>
          <LinkAction to={paths.pricing} trailing={<ArrowRight size={14} />}>
            Compare plans
          </LinkAction>
        </Actions>
      </div>
    </Section>
  );
}
