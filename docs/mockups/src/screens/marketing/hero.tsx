// The hero of the landing page: the outcome as the title, how ciproxy gets
// there in one sentence each, the two actions, and a picture of the product:
// the live queue of mokly-ai at a release peak. The backdrop is the grid on the
// landing page; the hero-backgrounds variations try the others.

import type { Viewport } from "@mokly/mokly";

import { LinkAction } from "../../components/actions.tsx";
import { paths } from "../../components/paths.ts";
import { freePlanNote } from "../../fixtures/tiers.ts";
import { HeroBackdropArt, type HeroBackdrop } from "./hero-backdrops.tsx";
import { HeroPanel } from "./hero-panel.tsx";
import { Actions } from "./section.tsx";
import "./hero.css";

export function Hero({
  viewport,
  backdrop = "grid",
}: {
  viewport: Viewport;
  backdrop?: HeroBackdrop;
}) {
  // On the black band, the white button leads and the second one is light.
  const onInk = backdrop === "ink";
  return (
    <section className={`hero hero-bg-${backdrop}`}>
      <div className="hero-copy">
        <HeroBackdropArt backdrop={backdrop} viewport={viewport} />
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
          <LinkAction
            to={paths.signIn}
            tone={onInk ? "secondary" : "primary"}
            size="lg"
          >
            Start free
          </LinkAction>
          <LinkAction
            to={paths.docsHome}
            tone={onInk ? "onMedia" : "secondary"}
            size="lg"
          >
            Read the docs
          </LinkAction>
        </Actions>
        <p className="caption hero-note">{freePlanNote}</p>
      </div>
      <HeroPanel viewport={viewport} />
    </section>
  );
}
