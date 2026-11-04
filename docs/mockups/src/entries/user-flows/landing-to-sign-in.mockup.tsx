import { defineUseCase } from "@mokly/mokly";

import { paths } from "../../components/paths.ts";
import { planDoc } from "../../components/state-schema.ts";

export const landingToSignIn = defineUseCase({
  title: "A visitor goes from the landing page to sign-in",
  description:
    "A developer whose team pays too much for hosted runners finds ciproxy.dev, checks the setup in the quickstart, compares the plans, and signs in with GitHub.",
  relatedDocs: [planDoc],
  steps: [
    {
      screenPath: paths.landing,
      title: "The landing page",
      description:
        "The hero leads with the outcome, and a real month of CI shows the cost. Follow the quickstart, under How it works, opens the docs.",
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
      description:
        "Free covers one machine and 2,000 starts a month, and every limit is soft. Start free opens sign-in.",
    },
    {
      screenPath: paths.signIn,
      title: "Sign in with GitHub",
      description:
        "GitHub signs the visitor in; ciproxy has no password of its own. The flow From sign-in to the first green job continues from here.",
    },
  ],
});
