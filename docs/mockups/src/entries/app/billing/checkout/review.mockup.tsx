import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { CheckoutReviewScreen } from "../../../../screens/billing/checkout-review.tsx";

export const review = defineScreen({
  title: "Review the plan",
  description:
    "The customer sets the add-on quantities and sees the monthly total before Stripe Checkout opens. Above Continue to payment, a line says that paid plans are for business use only, with a link to the Terms.",
  rationale:
    "The agent call of 22 November 2026, under the user's delegation: paid plans are for business use only. So each review with a pay button says so near the button, with a link to the Terms.",
  relatedDocs: related("billing", "metering", "termsAcceptance"),
  address: orgAddress("billing/checkout"),
  useCasePaths: [paths.flowFreeToTeam],
  render: (_props, context) => (
    <CheckoutReviewScreen viewport={context.viewport} move="to-team" />
  ),
});
