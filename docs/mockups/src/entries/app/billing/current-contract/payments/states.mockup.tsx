import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../../components/paths.ts";
import { related } from "../../../../../components/related-docs.ts";
import { orgAddress } from "../../../../../fixtures/org.ts";
import { PlanUsageScreen } from "../../../../../screens/billing/plan-usage.tsx";

const address = orgAddress("billing");

export const paymentFailed = defineScreen({
  slug: "payment-failed",
  title: "A failed payment, as the API words it",
  description:
    "Stripe could not charge the card on 4 December and tries again on 7 December. The API gives these dates but not the plan, so the banner says that your plan stays active. Update card opens the Stripe billing portal.",
  relatedDocs: related("billing", "security"),
  address,
  render: (_props, context) => (
    <PlanUsageScreen
      viewport={context.viewport}
      state="payment-failed"
      api={{ plans: paths.plansOnTeam }}
    />
  ),
});

export const movedToFree = defineScreen({
  slug: "moved-to-free",
  title: "Moved to Free, as the API words it",
  description:
    "Stripe ended the subscription after its retries, and mokly-ai is on Free. The API gives neither the plan that ended nor the day, so the banner says that your paid plan ended after failed payments. Choose a plan opens the plans. The pace notice names Team, the offer of the API.",
  relatedDocs: related("billing", "security"),
  address,
  render: (_props, context) => (
    <PlanUsageScreen
      viewport={context.viewport}
      state="moved-to-free"
      api={{ plans: paths.plans }}
    />
  ),
});
