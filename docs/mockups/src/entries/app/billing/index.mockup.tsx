import { defineScreen } from "@mokly/mokly";

import { planDoc } from "../../../components/state-schema.ts";
import { orgAddress } from "../../../fixtures/org.ts";
import { PlanUsageScreen } from "../../../screens/billing/plan-usage.tsx";

export const billing = defineScreen({
  title: "Plan and usage",
  description:
    "mokly-ai on Team in mid-November: 8,640 of 25,000 starts and 3 of 3 own machines this period, no add-ons, and the way to the Stripe billing portal for invoices and the card.",
  rationale:
    "The period follows the Stripe subscription, from the 4th to the 3rd, so the starts reset on the day Stripe charges.",
  relatedDocs: [planDoc],
  address: orgAddress("billing"),
  render: (_props, context) => (
    <PlanUsageScreen viewport={context.viewport} state="steady" />
  ),
});
