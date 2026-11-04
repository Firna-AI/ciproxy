import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { CheckoutReviewScreen } from "../../../../screens/billing/checkout-review.tsx";
import { PlanUsageScreen } from "../../../../screens/billing/plan-usage.tsx";
import { PlansScreen } from "../../../../screens/billing/plans.tsx";

export const toBusiness = defineScreen({
  slug: "to-business",
  title: "Move between paid tiers",
  description:
    "mokly-ai moves from Team to Business on 18 November. The move takes effect at once and keeps the period, so the 8,640 starts so far count toward Business. Stripe charges the card on file $133.33 for the 16 days left, with no Stripe Checkout.",
  relatedDocs: related("billing", "metering"),
  address: orgAddress("billing/change/business"),
  render: (_props, context) => (
    <CheckoutReviewScreen viewport={context.viewport} move="to-business" />
  ),
});

export const moveToFree = defineScreen({
  slug: "move-to-free",
  title: "Move to Free",
  description:
    "Move to Free on the Plans page asks first. Team and its add-ons stay until the end of the period, then mokly-ai is on Free, and jobs keep running on all three machines.",
  relatedDocs: related("billing", "metering"),
  address: orgAddress("billing/plans"),
  render: (_props, context) => (
    <PlansScreen viewport={context.viewport} state="move-to-free" />
  ),
});

export const freeScheduled = defineScreen({
  slug: "free-scheduled",
  title: "Team ends at the end of the period",
  description:
    "After Move to Free, Plan and usage shows the day Team ends. Until then, Keep Team opens the Stripe billing portal, which undoes the move.",
  relatedDocs: related("billing", "metering"),
  address: orgAddress("billing"),
  render: (_props, context) => (
    <PlanUsageScreen viewport={context.viewport} state="free-scheduled" />
  ),
});
