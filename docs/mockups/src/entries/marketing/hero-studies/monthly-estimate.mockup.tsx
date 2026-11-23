import { defineScreen } from "@mokly/mokly";

import { relatedMvp } from "../../../components/related-docs.ts";
import { siteAddress } from "../../../fixtures/site.ts";
import { MonthlyEstimate } from "../../../screens/marketing/hero-studies/monthly-estimate.tsx";
import { LandingScreen } from "../../../screens/marketing/landing.tsx";

export const monthlyEstimate = defineScreen({
  title: "Monthly estimate",
  description:
    "A paper estimate breaks the cost of one busy month into the machine and the ciproxy plan.",
  rationale:
    "A copper folder and a printed slip put the comparison and its assumptions beside the promise.",
  relatedDocs: relatedMvp("architecture", "billing", "metering"),
  address: siteAddress(),
  render: (_props, context) => (
    <LandingScreen viewport={context.viewport} hero={<MonthlyEstimate />} />
  ),
});
