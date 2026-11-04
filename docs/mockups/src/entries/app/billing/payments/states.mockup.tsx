import { defineScreen } from "@mokly/mokly";

import { planDoc } from "../../../../components/state-schema.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { PlanUsageScreen } from "../../../../screens/billing/plan-usage.tsx";

const address = orgAddress("billing");

export const paymentFailed = defineScreen({
  slug: "payment-failed",
  title: "A renewal payment failed",
  description:
    "Stripe could not charge the card for the December period. Team stays active while Stripe retries; Update card opens the Stripe billing portal.",
  relatedDocs: [planDoc],
  address,
  render: (_props, context) => (
    <PlanUsageScreen viewport={context.viewport} state="payment-failed" />
  ),
});

export const movedToFree = defineScreen({
  slug: "moved-to-free",
  title: "Moved to Free after failed payments",
  description:
    "Stripe ended the subscription after its retries on 18 December, so mokly-ai is on Free from that day, and the Free period counts only the starts since then. Its 3 own machines pass the Free limit; jobs keep running and the banner offers a plan.",
  relatedDocs: [planDoc],
  address,
  render: (_props, context) => (
    <PlanUsageScreen viewport={context.viewport} state="moved-to-free" />
  ),
});
