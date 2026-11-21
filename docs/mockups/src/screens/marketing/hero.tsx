// The hero of the landing page: the outcome as the title, how ciproxy gets
// there in one sentence each, the two actions, and a picture of the product:
// the live queue of mokly-ai at a release peak.

import type { Viewport } from "@mokly/mokly";

import { LinkAction } from "../../components/actions.tsx";
import { paths } from "../../components/paths.ts";
import { freePlanNote } from "../../fixtures/tiers.ts";
import { HeroPanel } from "./hero-panel.tsx";
import { Actions } from "./section.tsx";
import "./hero.css";

export function Hero({ viewport }: { viewport: Viewport }) {
  return (
    <section className="hero">
      <div className="hero-copy">
        <p className="hero-chip">
          <span className="hero-chip-label">One line in your workflow</span>
          <code className="hero-chip-code">runs-on: ciproxy</code>
        </p>
        <h1 className="hero-title">
          <span className="hero-line">Pay less for CI.</span>{" "}
          <span className="hero-line hero-soft">Keep your jobs fast.</span>
        </h1>
        <p className="hero-lead">
          ciproxy runs your GitHub Actions jobs on your own machines first, and
          starts runners in your Boat account only when they are full.
        </p>
        <Actions center>
          <LinkAction to={paths.signIn} tone="primary" size="lg">
            Start free
          </LinkAction>
          <LinkAction to={paths.docsHome} size="lg">
            Read the docs
          </LinkAction>
        </Actions>
        <p className="caption hero-note">{freePlanNote}</p>
      </div>
      <HeroPanel viewport={viewport} />
    </section>
  );
}
