import { defineScreen } from "@mokly/mokly";

import { paths } from "../../components/paths.ts";
import { relatedMvp } from "../../components/related-docs.ts";
import { siteAddress } from "../../fixtures/site.ts";
import { LandingScreen } from "../../screens/marketing/landing.tsx";

export const landing = defineScreen({
  title: "Landing page",
  description:
    "The home page of ciproxy.dev for a developer who pays too much for hosted runners. It leads with the outcome, shows the live queue at a release peak, the three setup steps, the features, the saving of one own machine against Blacksmith and GitHub-hosted runners, the tiers, and security. Every sign-in button says Start free.",
  rationale:
    "Graphite, with a larger title and tight tracking: the restraint of the Vercel and Linear sites. The only colour is the status dots of the product picture. The cost title claims a saving up to a percentage that the fixture computes from the list prices of one new Hetzner AX42, Blacksmith, and GitHub, checked on 4 November 2026. A small caption names the basis, with the average job of 5 minutes, and links to the calculation on the pricing page. The page shows no customer logos or quotes.",
  relatedDocs: relatedMvp("architecture", "billing", "metering", "security"),
  address: siteAddress(),
  useCasePaths: [paths.flowLandingToSignIn],
  render: (_props, context) => <LandingScreen viewport={context.viewport} />,
});
