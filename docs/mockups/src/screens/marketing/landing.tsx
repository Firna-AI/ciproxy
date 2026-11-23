// The landing page of ciproxy.dev: the hero with the outcome, how it works in
// three steps, the features, a real cost comparison, the pricing teaser,
// security, and the closing call to action. The hero backdrop is the grid,
// unless a variation of marketing/hero-backgrounds picks another.

import type { Viewport } from "@mokly/mokly";

import { ClosingCta } from "./closing-cta.tsx";
import { CostCompare } from "./cost-compare.tsx";
import { Features } from "./features.tsx";
import type { HeroBackdrop } from "./hero-backdrops.tsx";
import { Hero } from "./hero.tsx";
import { HowItWorks } from "./how-it-works.tsx";
import { MarketingShell } from "./marketing-shell.tsx";
import { PricingTeaser } from "./pricing-teaser.tsx";
import { Security } from "./security.tsx";
import "./landing.css";

export function LandingScreen({
  viewport,
  heroBackdrop,
}: {
  viewport: Viewport;
  heroBackdrop?: HeroBackdrop;
}) {
  return (
    <MarketingShell viewport={viewport} section="home">
      <Hero viewport={viewport} backdrop={heroBackdrop} />
      <HowItWorks />
      <Features />
      <CostCompare />
      <PricingTeaser />
      <Security />
      <ClosingCta />
    </MarketingShell>
  );
}
