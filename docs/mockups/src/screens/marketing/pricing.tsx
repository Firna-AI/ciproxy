// The pricing page of ciproxy.dev: the three tiers, the add-ons, the feature
// comparison, the common questions, and the closing call to action. Every
// price and limit comes from the approved pricing in fixtures/tiers.ts.

import type { Viewport } from "@mokly/mokly";

import { formatCount, formatMoney } from "../../fixtures/format.ts";
import { addOnPrices, tiers } from "../../fixtures/tiers.ts";
import { ClosingCta } from "./closing-cta.tsx";
import { MarketingShell } from "./marketing-shell.tsx";
import { PlanCompare } from "./plan-compare.tsx";
import { PriceCards } from "./price-cards.tsx";
import { PricingFaq } from "./pricing-faq.tsx";
import { Cells, Section, SectionHead } from "./section.tsx";
import "./pricing.css";

const addOns: readonly { price: number; title: string; text: string }[] = [
  {
    price: addOnPrices.startsPrice,
    title: `${formatCount(addOnPrices.startsBlock)} more starts`,
    text: "For each block, every month, on top of the starts of your plan.",
  },
  {
    price: addOnPrices.machinePrice,
    title: "One more own machine",
    text: "For each machine above the machines of your plan, every month.",
  },
];

function AddOns() {
  return (
    <Section flush>
      <SectionHead
        title="Add-ons"
        lead={`On ${tiers.team.name} and ${tiers.business.name}, add starts and machines by the month. Set the quantities in the dashboard; ciproxy adds no charges by itself.`}
      />
      <Cells columns={2}>
        {addOns.map((addOn) => (
          <div key={addOn.title} className="mk-cell addon">
            <p className="addon-price">
              <span className="addon-amount num">
                {formatMoney(addOn.price)}
              </span>
              <span className="muted">a month</span>
            </p>
            <h3 className="addon-title">{addOn.title}</h3>
            <p className="addon-text">{addOn.text}</p>
          </div>
        ))}
      </Cells>
    </Section>
  );
}

export function PricingScreen({ viewport }: { viewport: Viewport }) {
  return (
    <MarketingShell viewport={viewport} section="pricing">
      <Section flush className="pricing-top">
        <SectionHead
          page
          center
          eyebrow="Pricing"
          title="Priced by jobs, not by minutes"
          lead="Every plan runs your jobs on your own machines first, and on Boat when they are full. Limits are soft, so your jobs keep running past them."
        />
        <PriceCards />
        <p className="mk-foot caption pricing-note">
          Prices in US dollars, billed monthly through Stripe. Every workspace
          starts on Free and can change its plan in the dashboard at any time.
        </p>
      </Section>
      <AddOns />
      <PlanCompare />
      <PricingFaq />
      <ClosingCta />
    </MarketingShell>
  );
}
