import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { CheckoutReviewScreen } from "../../../../screens/billing/checkout-review.tsx";

export const review = defineScreen({
  title: "Review the plan",
  description:
    "The customer sets the add-on quantities and sees the monthly total before Stripe Checkout opens.",
  relatedDocs: related("billing", "metering"),
  address: orgAddress("billing/checkout"),
  useCasePaths: [paths.flowFreeToTeam],
  render: (_props, context) => (
    <CheckoutReviewScreen viewport={context.viewport} move="to-team" />
  ),
});
