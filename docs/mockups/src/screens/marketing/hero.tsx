// The hero of the landing page: the outcome as the title, how ciproxy gets
// there in one sentence each, the two actions, and a picture of the product:
// the live queue of mokly-ai at a release peak. The backdrop is the grid on the
// landing page; the hero-backgrounds variations try the others, and the
// hero-options variations reuse the text with other pictures.

import type { ReactNode } from "react";
import type { Viewport } from "@mokly/mokly";

import { LinkAction } from "../../components/actions.tsx";
import { paths } from "../../components/paths.ts";
import { freePlanNote } from "../../fixtures/tiers.ts";
import { HeroBackdropArt, type HeroBackdrop } from "./hero-backdrops.tsx";
import { HeroPanel } from "./hero-panel.tsx";
import { Actions } from "./section.tsx";
import "./hero.css";

/**
 * The text of the hero: the chip, the title, the lead, the two actions, and
 * the note of Free. On a black band, the white button leads.
 */
export function HeroText({
  center = true,
  onInk = false,
}: {
  center?: boolean;
  onInk?: boolean;
}) {
  return (
    <>
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
      <Actions center={center}>
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
    </>
  );
}

export function Hero({
  viewport,
  backdrop = "grid",
  picture,
}: {
  viewport: Viewport;
  backdrop?: HeroBackdrop;
  /** The picture under the text, in place of the live queue. */
  picture?: ReactNode;
}) {
  return (
    <section className={`hero hero-bg-${backdrop}`}>
      <div className="hero-copy">
        <HeroBackdropArt backdrop={backdrop} viewport={viewport} />
        <HeroText onInk={backdrop === "ink"} />
      </div>
      {picture ?? <HeroPanel viewport={viewport} />}
    </section>
  );
}
