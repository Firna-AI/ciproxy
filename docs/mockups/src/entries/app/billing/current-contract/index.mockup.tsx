import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { PlanUsageScreen } from "../../../../screens/billing/plan-usage.tsx";

export const currentContract = defineScreen({
  title: "Facts of the current API",
  description:
    "Plan and usage of mokly-ai on Team in mid-November, past the 25,000 starts of Team, as the API gives it today. The banner says that mokly-ai passed a limit of its plan and names the offer of the API, Team with 1 block of starts for $59 a month; Compare plans opens the plans. The add-on quantities carry no prices. The total is that of the saved add-ons, and Use the add-ons that cover your use sets the offer.",
  rationale:
    "The known pages stay as the target. These pages show only what the API gives today: no plan or passed limit in a banner, no day on which a paid plan ended, no unit prices of the add-ons, no price of a move between paid tiers yet, no receipt address, and no workspace in a Stripe return.",
  relatedDocs: related("billing", "metering"),
  address: orgAddress("billing"),
  render: (_props, context) => (
    <PlanUsageScreen
      viewport={context.viewport}
      state="over-starts"
      api={{ plans: paths.plansOnTeam }}
    />
  ),
});
