import { defineScreen } from "@mokly/mokly";

import { relatedMvp } from "../../../components/related-docs.ts";
import { siteAddress } from "../../../fixtures/site.ts";
import { ReleaseBoard } from "../../../screens/marketing/hero-studies/release-board.tsx";
import { LandingScreen } from "../../../screens/marketing/landing.tsx";

export const releaseBoard = defineScreen({
  title: "Release board",
  description:
    "A job board switches between the release peak and the afternoon. It shows how many jobs run on the machines and on Boat.",
  rationale:
    "Pine and signal green give the job counts the look of a station display. Two example jobs identify their destinations.",
  relatedDocs: relatedMvp("architecture", "billing", "metering"),
  address: siteAddress(),
  render: (_props, context) => (
    <LandingScreen viewport={context.viewport} hero={<ReleaseBoard />} />
  ),
});
