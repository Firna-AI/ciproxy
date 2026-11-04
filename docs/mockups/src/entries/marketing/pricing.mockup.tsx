import { defineScreen } from "@mokly/mokly";

import { paths } from "../../components/paths.ts";
import { related } from "../../components/related-docs.ts";
import { siteAddress } from "../../fixtures/site.ts";
import { PricingScreen } from "../../screens/marketing/pricing.tsx";

export const pricing = defineScreen({
  title: "Pricing",
  description:
    "The three tiers with their prices, starts, and own machines, the two add-ons, a feature comparison, and the common questions: what a start is, what counts as a machine, what happens past a limit, who pays for overflow, how to cancel, and how the landing page calculates its saving. Each feature that ships after the launch says Coming later, on the cards and in the comparison.",
  rationale:
    "Every button says Start free and opens sign-in, because every organization starts on Free and picks a paid tier in the dashboard. Team carries the black button without a claim such as Most popular, which ciproxy cannot make before launch. Alerts by email and Slack, single sign-on, the audit log, and custom VM images have no screen yet, so they say Coming later.",
  relatedDocs: related("billing", "metering"),
  address: siteAddress("pricing"),
  useCasePaths: [paths.flowLandingToSignIn],
  render: (_props, context) => <PricingScreen viewport={context.viewport} />,
});
