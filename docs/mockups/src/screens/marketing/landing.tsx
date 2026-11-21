// The landing page of ciproxy.dev: the hero with the outcome, how it works in
// three steps, the features, a real cost comparison, the pricing teaser,
// security, and the closing call to action.

import type { Viewport } from "@mokly/mokly";

import { ClosingCta } from "./closing-cta.tsx";
import { CostCompare } from "./cost-compare.tsx";
import { Features } from "./features.tsx";
import { Hero } from "./hero.tsx";
import { HowItWorks } from "./how-it-works.tsx";
import { MarketingShell } from "./marketing-shell.tsx";
import { PricingTeaser } from "./pricing-teaser.tsx";
import { Security } from "./security.tsx";
import "./landing.css";

export function LandingScreen({ viewport }: { viewport: Viewport }) {
  return (
    <MarketingShell viewport={viewport} section="home">
      <Hero viewport={viewport} />
      <HowItWorks />
      <Features />
      <CostCompare />
      <PricingTeaser />
      <Security />
      <ClosingCta />
    </MarketingShell>
  );
}
