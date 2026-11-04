import { defineUseCase } from "@mokly/mokly";

import { paths } from "../../components/paths.ts";
import { related } from "../../components/related-docs.ts";
import { costClaim } from "../../fixtures/cost-claim.ts";
import { freePlanNote } from "../../fixtures/tiers.ts";

export const landingToSignIn = defineUseCase({
  title: "A visitor goes from the landing page to sign-in",
  description:
    "A developer whose team pays too much for hosted runners finds ciproxy.dev, checks the setup in the quickstart, compares the plans, and signs in with GitHub.",
  relatedDocs: related("architecture", "billing", "security"),
  steps: [
    {
      screenPath: paths.landing,
      title: "The landing page",
      description: `The hero leads with the outcome, and the cost section claims a saving of up to ${costClaim.upToPercent}% with one own machine. Its caption links to the calculation. Follow the quickstart, under How it works, opens the docs.`,
    },
    {
      screenPath: paths.quickstart,
      title: "Read the quickstart",
      description:
        "One command for each machine and one line for each workflow. Pricing in the top bar opens the plans.",
    },
    {
      screenPath: paths.pricing,
      title: "Compare the plans",
      description: `${freePlanNote} Every limit is soft, and every tier says Start free, which opens sign-in.`,
    },
    {
      screenPath: paths.signIn,
      title: "Sign in with GitHub",
      description:
        "GitHub signs the visitor in; ciproxy has no password of its own. The flow From sign-in to the first green job continues from here.",
    },
  ],
});
