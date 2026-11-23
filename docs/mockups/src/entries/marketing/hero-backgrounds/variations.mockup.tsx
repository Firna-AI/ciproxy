import { defineScreen } from "@mokly/mokly";

import { relatedMvp } from "../../../components/related-docs.ts";
import { siteAddress } from "../../../fixtures/site.ts";
import type { HeroBackdrop } from "../../../screens/marketing/hero-backdrops.tsx";
import { LandingScreen } from "../../../screens/marketing/landing.tsx";

const address = siteAddress();
const relatedDocs = relatedMvp("architecture");

/** The landing page with one hero backdrop in place of the grid. */
function variation(
  slug: string,
  title: string,
  description: string,
  rationale: string,
  backdrop: HeroBackdrop,
) {
  return defineScreen({
    slug,
    title,
    description,
    rationale,
    relatedDocs,
    address,
    render: (_props, context) => (
      <LandingScreen viewport={context.viewport} heroBackdrop={backdrop} />
    ),
  });
}

export const plain = variation(
  "plain",
  "Plain white",
  "The hero with no background: white between the rails, the title, and the product panel.",
  "The type and the panel carry the hero alone, and the panel shadow gives the only depth. It is the quietest choice and keeps the page closest to the dashboard.",
  "plain",
);

export const ink = variation(
  "ink",
  "Black band",
  "The hero copy in white on a black band. The band ends behind the product panel, so the panel sits half on black and half on white.",
  "Black is the one accent of Graphite, so a black band is on brand and needs no new colour. It gives the page a strong start without a texture. On black, Start free takes the white button, and Read the docs the light button of Firna for dark backgrounds.",
  "ink",
);

export const workflow = variation(
  "workflow",
  "The workflow file",
  "The workflow file of mokly-ai in faint monospace at both sides of the title, with each runs-on: ciproxy line darker. A phone and a narrow window hide it.",
  "The texture is the product: the one line of the chip repeats in a real file. The lines come from fixtures/workflow.ts, beside the diff of the docs.",
  "workflow",
);

export const starts = variation(
  "starts",
  "Starts of 30 days",
  "The starts of mokly-ai for each of the last 30 days as faint bars behind the copy: own machines solid, Boat hatched above them. The release peak is the tallest bar.",
  "The texture is real data from the Savings report, fixtures/savings.ts, so it shows the idea of ciproxy: own machines take the base, and Boat takes the peaks.",
  "starts",
);

export const rule = variation(
  "rule",
  "One rule",
  "One hairline from rail to rail through the chip, with a cross where it meets each rail.",
  "It keeps the hairline frame of the page and drops the fading grid: one line in the workflow, one line in the picture.",
  "rule",
);
