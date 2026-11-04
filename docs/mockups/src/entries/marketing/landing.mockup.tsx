import { defineScreen } from "@mokly/mokly";

import { paths } from "../../components/paths.ts";
import { planDoc } from "../../components/state-schema.ts";
import { siteAddress } from "../../fixtures/site.ts";
import { LandingScreen } from "../../screens/marketing/landing.tsx";

export const landing = defineScreen({
  title: "Landing page",
  description:
    "The home page of ciproxy.dev for a developer who pays too much for hosted runners. It leads with the outcome, shows the live queue at a release peak, the three setup steps, the features, a real month of mokly-ai CI against Blacksmith and GitHub-hosted runners, the tiers, and security.",
  rationale:
    "Graphite, with a larger title and tight tracking: the restraint of the Vercel and Linear sites. The only colour is the status dots of the product picture. Every number comes from the dashboard fixtures, and the page shows no customer logos or quotes.",
  relatedDocs: [planDoc],
  address: siteAddress(),
  useCasePaths: [paths.flowLandingToSignIn],
  render: (_props, context) => <LandingScreen viewport={context.viewport} />,
});
