import type { ReactNode } from "react";
import { defineScreen, type Viewport } from "@mokly/mokly";

import { relatedMvp } from "../../../components/related-docs.ts";
import { siteAddress } from "../../../fixtures/site.ts";
import { Hero } from "../../../screens/marketing/hero.tsx";
import {
  EstimateCard,
  MachinesCard,
  PeakBoard,
} from "../../../screens/marketing/hero-options/pictures.tsx";
import { SplitHero } from "../../../screens/marketing/hero-options/split-hero.tsx";
import { LandingScreen } from "../../../screens/marketing/landing.tsx";

const address = siteAddress();
const relatedDocs = relatedMvp("architecture", "jobRouting", "overflow");

/** The landing page with one hero option in place of the landing hero. */
function option(
  slug: string,
  title: string,
  description: string,
  rationale: string,
  hero: (viewport: Viewport) => ReactNode,
) {
  return defineScreen({
    slug,
    title,
    description,
    rationale,
    relatedDocs,
    address,
    render: (_props, context) => (
      <LandingScreen
        viewport={context.viewport}
        hero={hero(context.viewport)}
      />
    ),
  });
}

export const machines = option(
  "machines",
  "Text left, machines right",
  "The hero text on the left and a card of the three machines of mokly-ai on the right: every slot busy, 46 jobs on Boat, and 12 that wait. The grid is gone.",
  "The smallest step from the landing page: the same words, the same Graphite look, and a picture of the product beside the text in place of decoration. The split layout comes from the hero studies of PR #136.",
  () => <SplitHero picture={<MachinesCard />} />,
);

export const drawing = option(
  "drawing",
  "Machines with a drawing",
  "The split hero with a line drawing of the three AX42 machines at the top of the machines card, in ink.",
  "The drawing of the Machine room study of PR #136, in ink and greys in place of cobalt. It shows real hardware, which no other CI site shows, and keeps the look of the site.",
  () => <SplitHero picture={<MachinesCard drawing />} />,
);

export const accent = option(
  "accent",
  "Machines with one accent",
  "The drawing option with one blue accent: the drawing, the chip code, and the second title line.",
  "One colour in place of the eight of PR #136. Graphite keeps colour for status today, so this option adds a proposed accent to @ciproxy/tokens. The buttons stay black.",
  () => <SplitHero accent picture={<MachinesCard drawing />} />,
);

export const estimate = option(
  "estimate",
  "Text left, monthly estimate right",
  "The split hero with a monthly estimate: one busy AX42 and the ciproxy plan against the cheaper hosted runners, with the saving and a link to the calculation.",
  "The receipt of the Monthly estimate study of PR #136, straight and in Graphite. Every price comes from the cost claim of the landing page, so it backs the title with the same numbers.",
  () => <SplitHero picture={<EstimateCard />} />,
);

export const board = option(
  "board",
  "Centred, with a peak board",
  "The centred hero of the landing page without the grid, over a board of the release peak: 18 jobs on your machines, 46 on Boat, and 12 that wait, with the first running jobs.",
  "It keeps the layout of the landing page and makes the product panel the picture: the three counts of the Release board study of PR #136 read at a glance, in Graphite.",
  (viewport) => (
    <Hero
      viewport={viewport}
      backdrop="plain"
      picture={<PeakBoard rows={viewport === "mobile" ? 3 : 4} />}
    />
  ),
);
