import { defineScreen } from "@mokly/mokly";

import { paths } from "../../components/paths.ts";
import { planDoc } from "../../components/state-schema.ts";
import { siteAddress } from "../../fixtures/site.ts";
import { PricingScreen } from "../../screens/marketing/pricing.tsx";

export const pricing = defineScreen({
  title: "Pricing",
  description:
    "The three tiers with their prices, starts, and own machines, the two add-ons, a feature comparison, and the common questions: what a start is, what counts as a machine, what happens past a limit, who pays for overflow, and how to cancel.",
  rationale:
    "Every action opens sign-in, because every organization starts on Free and picks a paid tier in the dashboard. Team carries the black button without a claim such as Most popular, which ciproxy cannot make before launch.",
  relatedDocs: [planDoc],
  address: siteAddress("pricing"),
  useCasePaths: [paths.flowLandingToSignIn],
  render: (_props, context) => <PricingScreen viewport={context.viewport} />,
});
