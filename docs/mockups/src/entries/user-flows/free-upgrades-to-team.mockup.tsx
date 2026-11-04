import { defineUseCase } from "@mokly/mokly";

import { paths } from "../../components/paths.ts";
import { related } from "../../components/related-docs.ts";

export const freeUpgradesToTeam = defineUseCase({
  title: "A Free organization passes its starts and upgrades to Team",
  description:
    "mokly-ai runs about 600 jobs a day, so on 4 September it passes the 2,000 starts of Free. Jobs keep running; the dashboard offers Team, and Cal pays through Stripe Checkout.",
  relatedDocs: related("metering", "billing"),
  steps: [
    {
      screenPath: paths.jobsStartsPassed,
      title: "The banner on Jobs",
      description:
        "Jobs keep running past the Free limit. A banner on every page says so and offers Team.",
    },
    {
      screenPath: paths.usageFreeOver,
      title: "Plan and usage",
      description:
        "2,214 of 2,000 starts, and Team covers the pace of this month.",
    },
    {
      screenPath: paths.plans,
      title: "Compare the plans",
      description: "Team carries the Covers your use badge. Cal chooses Team.",
    },
    {
      screenPath: paths.checkoutReview,
      title: "Review the plan",
      description: "No add-ons are needed. The total is $49 a month.",
    },
    {
      screenPath: paths.checkoutStripe,
      title: "Pay on Stripe",
      description: "Stripe Checkout takes the card; ciproxy never sees it.",
    },
    {
      screenPath: paths.checkoutConfirming,
      title: "Back from Stripe",
      description:
        "The return page waits a few seconds for the payment confirmation.",
    },
    {
      screenPath: paths.checkoutConfirmed,
      title: "mokly-ai is on Team",
      description:
        "The first Team period starts today, so its starts count from zero against 25,000. Add machines leads to the next own machine.",
    },
  ],
});
